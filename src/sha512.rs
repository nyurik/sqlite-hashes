use sha2::Sha512;

use crate::rusqlite::{Connection, Result};

pub fn register_sha512_function(conn: &Connection) -> Result<()> {
    crate::core::create_hash_fn::<Sha512>(conn, "sha512")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::test::{hex, is_err, is_null, txt};

    #[test]
    fn test_sha512() {
        let db = Connection::open_in_memory().unwrap();
        let c = &db;
        register_sha512_function(c).unwrap();

        is_null(c, "sha512(NULL)");
        is_err(c, "sha512(1)");
        is_err(c, "sha512(0.42)");

        hex(
            c,
            "sha512('')",
            "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
        );
        hex(
            c,
            "sha512('a')",
            "1f40fc92da241694750979ee6cf582f2d5d7d28e18335de05abc54d0560e0f5302860c652bf08d560252aa5e74210546f369fbbbce8c12cfc7957b2652fe9a75",
        );
        hex(
            c,
            "sha512('123456789')",
            "d9e6762dd1c8eaf6d61b3c6192fc408d4d6d5f1176d0c29169bc24e71c3f274ad27fcd5811b313d681f7e55ec02d73d499c95455b6b5bb503acf574fba8ffe85",
        );
        hex(
            c,
            "sha512(x'')",
            "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
        );
        hex(
            c,
            "sha512(x'00')",
            "b8244d028981d693af7b456af8efa4cad63d282e19ff14942c246e50d9351d22704a802a71c3580b6370de4ceb293c324a8423342557d4e5c38438f0e36910ee",
        );
        hex(
            c,
            "sha512(x'0123456789abcdef')",
            "650161856da7d9f818e6047cf6b2092bc7aa3767d3495cfbefe2b710ed684a43ba933ea8286ef67d975e64e0482e5ebe0701788989396545b6badb3b0a136f19",
        );

        txt(
            c,
            "hex(sha512(''))",
            "CF83E1357EEFB8BDF1542850D66D8007D620E4050B5715DC83F4A921D36CE9CE47D0D13C5D85F2B0FF8318D2877EEC2F63B931BD47417A81A538327AF927DA3E",
        );
        txt(
            c,
            "hex(sha512('a'))",
            "1F40FC92DA241694750979EE6CF582F2D5D7D28E18335DE05ABC54D0560E0F5302860C652BF08D560252AA5E74210546F369FBBBCE8C12CFC7957B2652FE9A75",
        );
        txt(
            c,
            "hex(sha512(x'00'))",
            "B8244D028981D693AF7B456AF8EFA4CAD63D282E19FF14942C246E50D9351D22704A802A71C3580B6370DE4CEB293C324A8423342557D4E5C38438F0E36910EE",
        );
    }
}
