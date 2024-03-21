use std::{io::Cursor, str::from_utf8};

use base64::{engine::general_purpose, Engine as _};
use log::trace;
use pgp::composed::{Deserializable, Message, SignedPublicKey};

use crate::hash::read_short_hash;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct AuthName {
    pub name: String,
}

impl AuthName {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        AuthName {
            name: name.as_ref().to_owned(),
        }
    }

    pub fn parse<S: AsRef<str>>(login_cookie: S) -> Option<Self> {
        let msg = urlencoding::decode(login_cookie.as_ref()).unwrap();
        trace!("Cookie value: {}", msg);
        let mut auth_type = read_gpg(msg.as_ref());

        if auth_type.is_none() {
            let result = read_short_hash(msg.as_ref());

            if let Ok(result) = result {
                trace!("passing auth name result {}", result.as_ref().unwrap().name);
                auth_type = result;
            }
        }

        auth_type
    }
}

fn get_gpg_message(message: &str) -> Option<String> {
    if message.starts_with("-----BEGIN") {
        Some(message.to_string())
    } else {
        let decoded = general_purpose::STANDARD.decode(message.replace('\n', ""));
        if let Ok(decoded) = decoded {
            let parsed = from_utf8(&decoded);
            if let Ok(parsed) = parsed {
                Some(parsed.to_string())
            } else {
                eprintln!("Could not convert from UTF-8");
                None
            }
        } else {
            eprintln!("Could not convert from base64");
            None
        }
    }
}

pub fn verify_gpg(message: &str) -> Result<(), pgp::errors::Error> {
    let message = get_gpg_message(message);
    if let Some(message) = message {
        let (msg, _) = Message::from_armor_single(Cursor::new(&message.as_bytes()))?;
        let key = get_public_key();
        msg.verify(&key)
    } else {
        Err(pgp::errors::Error::InvalidInput)
    }
}

pub fn read_gpg(message: &str) -> Option<AuthName> {
    let verified = verify_gpg(message);
    if verified.is_err() {
        return None;
    }
    let message = get_gpg_message(message)?;
    let msg_res = Message::from_armor_single(Cursor::new(&message.as_bytes()));

    if let Ok((msg, _)) = msg_res {
        let read_res = msg.get_content();

        if let Ok(res) = read_res {
            match res {
                Some(res_bytes) => {
                    let parsed_res = from_utf8(&res_bytes[..]);
                    if let Ok(parsed) = parsed_res {
                        return Some(AuthName {
                            name: parsed.trim().to_string(),
                        });
                    }
                }
                _ => {
                    return None;
                }
            }
        }
    }
    None
}

pub fn get_public_key() -> SignedPublicKey {
    let key_bytes = include_bytes!("../pubkey.asc");
    SignedPublicKey::from_armor_single(Cursor::new(key_bytes))
        .expect("Failed to load key")
        .0
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn get_base64_message() -> String {
        let message = "
LS0tLS1CRUdJTiBQR1AgTUVTU0FHRS0tLS0tCgpvd0VCVWdLdC9aQU5Bd0FJQVRyc3dhWWVIQWlu
QWNzTFlnQmpueDJGZEdWemRBcUpBak1FQUFFSUFCMFdJUVMwCkVZaTNMaHF1Q3pTeFVTRTY3TUdt
SGh3SXB3VUNZNThkaFFBS0NSQTY3TUdtSGh3SXA1MkVFQUNDbU1Vc1hjNVkKM1kvU09YaHJ4aGs3
eDhaQTU4WG5pUkNHRmVHV2dHVkFBSW5oVWhpNkJrcVpQUEJEWHpOR0hsZWRFSk1LYThIQworQkdW
OEliQ2ZMb242c3dlY1lwWjk1TmpwRGJZcDRXYzBnRjEyTm1OS2lkRXFqVHdJMzROS2t1QllESTNm
ZzhpClVwS1BXTEdqZlB4dldmLzJLeGdUdzFVNkN3Yy9jZ1BvSGJrZ010Mkl0aWxSLzhCTzVoTSsz
Y3grZ2wwT2FPbU0KNzdCTHFLVk5lUjZ4c25TMGFqdHNwczVpT1d1ckxTSHlNOUNGaWZ0NWVWdm9k
TW51WHNKR2xxNnd4TUdCUGFIMQpQNHVPUjlub0krNW9qMHZjQW5kRnRTMmZOT3BaMkdPcy9oUFla
L1ovRXpNMlI3STRvdWxsbEhIaGNVdHJ3ZzdyCnlsbjlJZG1hTkl1dTZDajYrSGpGZFVEVnJqUnpj
ZVhDbEFrZ0JHVE5NbDZweDdSL2Q1ME9NZ2NVTURIRXNadkoKYk9RaWJKV1ltMjJEN1ZrbW9lWUxw
VjFUR1lIclF5NDVFVzBvOTlXNjZMeDN3K3lTYXZrNkZjdFFPR2NjdWZXWAoxV0hlRm5OQXA3MDRY
ZmF6L0dlLzI4RHhnWXlzOGI1OHVnTkR6M09DMmpWNlJiMG81L0lFN05teitrTXhIalcwCnVSR25S
eVZCNWJucm8xSVVNZ1JEZnVpcEZORWxjMG5HZzlzTno0Z21oN2VOQ1NDTzI5em11Q3c5ZnhJenI0
NHYKM2UvbmJrY0kzOEltQUtZRnhGdnRCSXpmWUlFNkZ1M0ZxNG52Wk1Zc05MbUNOZzVuQ1pXa2pU
RnB5eHBWcVhFSQphY29jSHg3MDhndVA5NzhDbHZmWHVxdTk4cmpSQ0dLZE1RPT0KPWhlWWIKLS0t
LS1FTkQgUEdQIE1FU1NBR0UtLS0tLQo=";
        message.to_string()
    }
    fn get_test_message() -> String {
        let message = "
-----BEGIN PGP MESSAGE-----

owEBUgKt/ZANAwAIATrswaYeHAinAcsLYgBjnx2ndGVzdAqJAjMEAAEIAB0WIQS0
EYi3LhquCzSxUSE67MGmHhwIpwUCY58dpwAKCRA67MGmHhwIpzv/D/0ZETV6V7sw
UlPuzbUzn2NAihAaEuVlyuhUZQrVbjBGzIhF8MeyfU41z2lR1Oi+ilZ6gQ1dS+FZ
cyr1rg51Gqic+pAtg7YlMg0CLZgyQF6iCcBqnZWlm3mxTntfZNbadwgLg5pDtQgR
saMzLc+Yg1ynTfZ9e78RRBYgak9PSh2oGCrL5S97nnDrPEH0Z/djR/A7vBiDVi4S
ttcOaZjwcu7H3kk93SFdJ1h5jGTINz70zB2dGgJHZjMX1radKAHcfWv2iJdJhikd
qsopZMPnSiirLOkpUSL7LLz2DcmNxP9asWBQH1UGK3WZYLIxNKwu5KusphbKEYPZ
rN8GYInribpIC5u0W6BoO+1qdeqP9CI/PRRajGuJKJ31fEIyfCqtcV4QhHKZJIGX
PR64xfEQPcmdkICDrXE6MtQhSz/Qk9ximaWoY/+X17DiqRYRNgWArpBgj8n1Uu/9
2u43EfbxANF6A7yTci6lyG5W9IuI3mYW8y+sqKHjpB9ZCkbG8klf1YzrDMWFaY95
kF3DuPY0/uGKtr0F5WXq71ZgYdm6ysHIVtPkd0Ovr9GNooxusG15a7QkvzGKZ3JX
nVfyRv0nbBLkWpDgoGjneAiTSWlU3YnqWh9cQPLGHKSrvYXHMl/LDkOvyR72BGn+
S5iOlQNu8SFUGn5pfAwdWKipHoA+fvlpRg==
=R7m3
-----END PGP MESSAGE-----";
        message.trim().to_string()
    }

    #[test]
    fn it_reads_content() {
        let message = get_test_message();
        let msg = read_gpg(&message);

        assert!(msg.is_some());
        assert_eq!("test", msg.unwrap().name);
    }
    #[test]
    fn it_reads_base64_content() {
        let message = get_base64_message();
        let msg = read_gpg(&message);

        assert!(msg.is_some());
        assert_eq!("test", msg.unwrap().name);
    }
}
