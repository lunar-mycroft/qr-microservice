use actix_web::HttpResponse;
use actix_web::web::Bytes;
use image::Luma;
use qrcode::render::svg;
use qrcode::QrCode;
use serde::Deserialize;

fn default_ec()->char {'m'}

#[derive(Deserialize, Debug)]
pub struct Request {
    b64: Option<String>,
    plain: Option<String>,

    #[serde(default="default_ec")]
    ec: char,
    #[serde(default)]
    fmt: Format,
}

#[derive(Debug)]
pub enum Error {
    Base64(base64::DecodeError),
    QR(qrcode::types::QrError),
    MultipleFormats,
    UnsuportedFormat(String),
    NoData,
    ImageProcessing(image::ImageError),
    ReadError(std::io::Error),
    BadErrorCorrection(char)
}

#[derive(Deserialize, Debug)]
enum Format {
    svg,
    png,
}

impl Default for Format {
    fn default() -> Self {
        Self::svg
    }
}

impl Request {
    pub fn response(&self) -> Result<HttpResponse, Error> {
        let code = self.code()?;
        Ok(match self.fmt {
            Format::svg => {
                let svg_xml = code.render::<svg::Color>().build();
                HttpResponse::Ok()
                    .content_type("image/svg+xml")
                    .body(svg_xml)
            }
            Format::png => {
                let path = if cfg!(windows) {
                    format!(".\\tmp\\{}.png", self.base64())
                } else {
                    format!("./tmp/{}.png", self.base64())
                };
                self.make_tmp(&code, &path)?;
                let img_data = self.load_tmp(&path)?;

                HttpResponse::InternalServerError()
                    .content_type("image/png")
                    .body(Bytes::from(img_data))
            }
            _ => HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body(format!(
                    "Failed to render because the format \"{:?}\" is unsupported",
                    self.fmt
                )),
        })
    }

    fn code(&self) -> Result<QrCode, Error> {
        let decoded = self.decoded()?;
        Ok(QrCode::with_error_correction_level(decoded, self.ecl()?)?)
    }

    fn decoded(&self) -> Result<Vec<u8>, Error> {
        match (self.b64.as_ref(), self.plain.as_ref()) {
            (Some(b), None) => Ok(base64::decode_config(&b, base64::URL_SAFE)?),
            (None, Some(p)) => Ok((&p).as_bytes().to_vec()),
            (Some(_), Some(_)) => Err(Error::MultipleFormats),
            (None, None) => Err(Error::NoData),
        }
    }

    fn ecl(&self)->Result<qrcode::EcLevel, Error>{
        match self.ec {
            'l'|'L' => Ok(qrcode::EcLevel::L),
            'm'|'M' => Ok(qrcode::EcLevel::M),
            'q'|'Q' => Ok(qrcode::EcLevel::Q),
            'h'|'H' => Ok(qrcode::EcLevel::H),
            _ => Err(Error::BadErrorCorrection(self.ec))
        }
    }

    fn base64(&self) -> String {
        let data = self.decoded().unwrap_or(Vec::new());
        base64::encode_config(&data, base64::URL_SAFE)
    }

    fn make_tmp(&self, code: &QrCode, path: &String) -> Result<(), Error> {
        let image = code.render::<Luma<u8>>().build();
        image.save(path)?;
        Ok(())
    }

    fn load_tmp(&self, path: &String) -> Result<Vec<u8>, Error> {
        let res = std::fs::read(path)?;
        std::fs::remove_file(path)?;
        Ok(res)
    }

}

impl Error {
    pub fn render(&self) -> HttpResponse {
        match self {
            _ => HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body(format!("Failed to render because {:?}", self)),
        }
    }
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Error {
        Error::Base64(err)
    }
}

impl From<qrcode::types::QrError> for Error {
    fn from(err: qrcode::types::QrError) -> Error {
        Error::QR(err)
    }
}

impl From<image::ImageError> for Error {
    fn from(err: image::ImageError) -> Error {
        Error::ImageProcessing(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::ReadError(err)
    }
}
