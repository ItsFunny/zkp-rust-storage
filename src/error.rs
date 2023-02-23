use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
pub use thiserror::Error;
// pub use failure::Error as FError;
use crate::error::ErrorEnums::Kind;

pub type ZKResult<T> = std::result::Result<T, ZKError>;


#[derive(Debug)]
pub struct ZKError {
    code: usize,
    msg: String,
    err: Option<Box<dyn Error>>,
    wrapped_error: Option<Box<ZKError>>,
}

unsafe impl Send for ZKError {}

unsafe impl Sync for ZKError {}

impl ZKError {
    pub fn get_code(&self) -> usize {
        self.code
    }
    pub fn get_msg(&self) -> &String {
        &self.msg
    }
    pub fn new(code: usize, msg: String) -> Self {
        ZKError { code, msg, err: None, wrapped_error: None }
    }
    pub fn with_wrapped_error(mut self, e: Box<ZKError>) -> Self {
        self.wrapped_error = Some(e);
        self
    }
    pub fn with_error(mut self, e: Box<dyn Error>) -> Self {
        self.err = Some(e);
        self
    }
}

impl From<&ErrorEnums> for ZKError {
    fn from(s: &ErrorEnums) -> Self {
        ZKError::new(s.get_code(), s.get_msg().to_string())
    }
}

impl From<&str> for ZKError {
    fn from(msg: &str) -> Self {
        ZKError::new(0, msg.to_string())
    }
}


// TODO , bad codes here
impl Display for ZKError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut msg = format!("code={},msg={}", self.code, self.msg);
        match &self.err {
            Some(e) => {
                msg.push_str(",err=");
                msg.push_str(e.to_string().as_str())
            }
            None => {}
        }
        match &self.wrapped_error {
            Some(v) => {
                msg.push_str(",wrapped err=");
                msg.push_str(v.to_string().as_str());
            }
            None => {}
        }
        write!(f, "{}", msg)
    }
}

impl Error for ZKError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.wrapped_error {
            Some(v) => {
                Some(v)
            }
            None => {
                None
            }
        }
    }
}


//// enums
// #[derive(Debug)]
// #[non_exhaustive]
pub enum ErrorEnums {
    Kind(usize, &'static str),
}

impl ErrorEnums {
    pub fn get_code(&self) -> usize {
        match self {
            Kind(code, msg) => {
                *code
            }
            _ => {
                0
            }
        }
    }
    pub fn get_msg(&self) -> &'static str {
        match self {
            Kind(code, msg) => {
                *msg
            }
            _ => {
                "wrong type"
            }
        }
    }
    pub fn is_success(&self) -> bool {
        self.get_code() == 0
    }
}



#[macro_export]
macro_rules! error_enums {
    (
       $(
            $(#[$docs:meta])*
             ($name:ident, $code:expr, $msg:expr);
        )+
    )=>{
        #[derive(Debug)]
        pub struct ErrorEnumsStruct {
        }
        impl ErrorEnumsStruct
        {
            $(
                pub const $name:&'static $crate::error::ErrorEnums=&$crate::error::Kind($code,$msg);
            )+
            // TODO ,register errors
        }
    }
}

error_enums!(
    (SUCCESS,0,"success");
    (UNKNOWN,1,"unknown");
    (IO_ERROR,2,"IO FAILED");
    (JSON_SERIALIZE,3,"json serialize failed");
    (RESPONSE_FAILED,4,"response failed");
    (COMMAND_NOT_EXISTS,5,"command not exists");
    (CHANNEL_SEND_FAILED,6,"channel send failed");
    (INTERNAL_SERVER_ERROR,7,"internal server error");
    (DUPLICATE_OPTION,7,"DUPLICATE_OPTION");
    (ILLEGAL_STEP,8,"ILLEGAL_STEP");
    (DUPLICATE_STEP,9,"DUPLICATE_STEP");
    (EVENT_BUS_DUPLICATE_CLIENTID,10,"duplicate client id");
    (EVENT_BUS_SUBSCRIBE_FAILED,11,"failed to subscribe");
);
