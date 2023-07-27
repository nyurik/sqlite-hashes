use sha2::Sha256;

use crate::rusqlite::{Connection, Result};

pub fn register_sha256_function(conn: &Connection) -> Result<()> {
    crate::core::create_hash_fn::<Sha256>(conn, "sha256")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::test::{hex, is_err, is_null, txt};

    #[test]
    fn test_sha256() {
        let db = Connection::open_in_memory().unwrap();
        let c = &db;
        register_sha256_function(c).unwrap();

        is_null(c, "sha256(NULL)");
        is_err(c, "sha256(1)");
        is_err(c, "sha256(0.42)");

        hex(
            c,
            "sha256('')",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        );
        hex(
            c,
            "sha256('a')",
            "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb",
        );
        hex(
            c,
            "sha256('123456789')",
            "15e2b0d3c33891ebb0f1ef609ec419420c20e320ce94c65fbc8c3312448eb225",
        );
        hex(
            c,
            "sha256(x'')",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        );
        hex(
            c,
            "sha256(x'00')",
            "6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d",
        );
        hex(
            c,
            "sha256(x'0123456789abcdef')",
            "55c53f5d490297900cefa825d0c8e8e9532ee8a118abe7d8570762cd38be9818",
        );

        txt(
            c,
            "hex(sha256(''))",
            "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
        );
        txt(
            c,
            "hex(sha256('a'))",
            "CA978112CA1BBDCAFAC231B39A23DC4DA786EFF8147C4E72B9807785AFEE48BB",
        );
        txt(
            c,
            "hex(sha256(x'00'))",
            "6E340B9CFFB37A989CA544E6BB780A2C78901D3FB33738768511A30617AFA01D",
        );
    }
}
