#[derive(Debug)]
pub struct Ec2Error {
    pub message: String,
}

impl std::fmt::Display for Ec2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Ec2Error {
    pub fn new(message: String) -> Ec2Error {
        Ec2Error { message }
    }
}

impl std::error::Error for Ec2Error {}
