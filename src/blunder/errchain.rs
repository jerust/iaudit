use std::error::Error;
use std::fmt::{Formatter, Result};

pub fn errorchain(error: &impl Error, formatter: &mut Formatter<'_>) -> Result {
    // 将exception.message中的信息引入到exception.details开头
    // writeln!(formatter, "{}\n", error)?;

    // 错误堆栈跟踪, 直到找到错误源头
    let mut current = error.source();
    while let Some(cause) = current {
        // 防止打印重复的错误
        if cause.to_string() == error.to_string() {
            break;
        }
        writeln!(formatter, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }

    Ok(())
}
