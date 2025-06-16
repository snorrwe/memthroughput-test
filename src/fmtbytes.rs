use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Bytes(pub f64);

impl Display for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0;
        if bytes < 1000.0 {
            write!(f, "{bytes:.2} bytes")
        } else if bytes < 1000e3 {
            write!(f, "{:.2} KB", bytes / 1e3)
        } else if bytes < 1000e6 {
            write!(f, "{:.2} MB", bytes / 1e6)
        } else if bytes < 1000e9 {
            write!(f, "{:.2} GB", bytes / 1e9)
        } else if bytes < 1000e12 {
            write!(f, "{:.2} TB", bytes / 1e12)
        } else {
            write!(f, "{bytes:.2} bytes")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_num_formatting() {
        let result = Bytes(13980987619.602848).to_string();

        assert_eq!(result, "13.98 GB");
    }
}
