#[derive(Debug)]
pub enum ErrorType {
    SaveLoad,
    Assets,
}

impl ::std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ErrorType::SaveLoad => "Save/Load",
            ErrorType::Assets => "Assets",
        })
    }
}

#[derive(Debug)]
pub struct InnerError {
    pub ty: ErrorType,
    pub line: u32,
    pub file: String,
    pub message: String,
    pub chained: Option<Box<InnerError>>,
}

impl ::std::fmt::Display for InnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(original) = self.chained.as_ref() {
            writeln!(f, "{}", original)?
        }

        write!(f, "[ERROR][{}:{}] {} - {}", self.file, self.line, self.ty, self.message)
    }
}

#[derive(Debug)]
pub struct Error {
    pub inner: Box<InnerError>
}

impl Error {
    #[cold]
    #[inline(never)]
    pub fn new(ty: ErrorType, file: &'static str, line: u32, message: String) -> Self {
        let inner = InnerError {
            ty,
            file: file.to_string(),
            line,
            message,
            chained: None,
        };

        Error {
            inner: Box::new(inner)
        }
    }

    pub fn merge(&mut self, mut other: Self) {
        ::std::mem::swap(self, &mut other);
        self.inner.chained = Some(other.inner);
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

macro_rules! error {
    ($ty:expr, $($arg:tt)*) => {{
        let message = format!($($arg)*);
        $crate::error::Error::new($ty, file!(), line!(), message)
    }};
}

macro_rules! save_err { ($($arg:tt)*) => { error!($crate::error::ErrorType::SaveLoad, $($arg)*) }; }
macro_rules! assets_err { ($($arg:tt)*) => { error!($crate::error::ErrorType::Assets, $($arg)*) }; }
