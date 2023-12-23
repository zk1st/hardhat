#[derive(Debug)]
pub struct CustomError {
    pub name: String,
    // Note: All we need from this model is ABI related, and we derive it from the
    //   contract's json abi
}
