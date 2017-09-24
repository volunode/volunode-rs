#[derive(Debug)]
pub struct CertSig {
    pub signature: String,
    pub subject: String,
    pub hash: String,
}

#[derive(Debug)]
pub struct CertSigs {
    pub signatures: Vec<CertSig>,
}
