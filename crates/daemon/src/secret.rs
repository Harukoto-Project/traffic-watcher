use anyhow::Context;

fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

fn from_hex(text: &str) -> anyhow::Result<Vec<u8>> {
    let text = text.trim();
    anyhow::ensure!(text.len().is_multiple_of(2), "invalid hex length");
    (0..text.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&text[i..i + 2], 16)
                .with_context(|| format!("invalid hex at offset {i}"))
        })
        .collect()
}

#[cfg(windows)]
mod dpapi {
    use std::ffi::c_void;

    use anyhow::Context;
    use windows::Win32::Foundation::{HLOCAL, LocalFree};
    use windows::Win32::Security::Cryptography::{
        CRYPT_INTEGER_BLOB, CRYPTPROTECT_LOCAL_MACHINE, CryptProtectData, CryptUnprotectData,
    };
    use windows::core::w;

    const ENTROPY: &[u8] = b"traffic-watcher/webhook/v1";

    fn blob(bytes: &[u8]) -> CRYPT_INTEGER_BLOB {
        CRYPT_INTEGER_BLOB {
            cbData: bytes.len() as u32,
            pbData: bytes.as_ptr() as *mut u8,
        }
    }

    fn take_blob(out: CRYPT_INTEGER_BLOB) -> Vec<u8> {
        let bytes = unsafe { std::slice::from_raw_parts(out.pbData, out.cbData as usize) }.to_vec();
        unsafe {
            let _ = LocalFree(Some(HLOCAL(out.pbData as *mut c_void)));
        }
        bytes
    }

    pub fn protect(plain: &[u8]) -> anyhow::Result<Vec<u8>> {
        let input = blob(plain);
        let entropy = blob(ENTROPY);
        let mut output = CRYPT_INTEGER_BLOB::default();
        unsafe {
            CryptProtectData(
                &input,
                w!("traffic-watcher"),
                Some(&entropy),
                None,
                None,
                CRYPTPROTECT_LOCAL_MACHINE,
                &mut output,
            )
        }
        .context("CryptProtectData failed")?;
        Ok(take_blob(output))
    }

    pub fn unprotect(cipher: &[u8]) -> anyhow::Result<Vec<u8>> {
        let input = blob(cipher);
        let entropy = blob(ENTROPY);
        let mut output = CRYPT_INTEGER_BLOB::default();
        unsafe { CryptUnprotectData(&input, None, Some(&entropy), None, None, 0, &mut output) }
            .context("CryptUnprotectData failed")?;
        Ok(take_blob(output))
    }
}

#[cfg(not(windows))]
mod dpapi {
    pub fn protect(_plain: &[u8]) -> anyhow::Result<Vec<u8>> {
        anyhow::bail!("DPAPI is only available on Windows")
    }

    pub fn unprotect(_cipher: &[u8]) -> anyhow::Result<Vec<u8>> {
        anyhow::bail!("DPAPI is only available on Windows")
    }
}

pub fn protect(plain: &str) -> anyhow::Result<String> {
    Ok(to_hex(&dpapi::protect(plain.as_bytes())?))
}

pub fn unprotect(hex: &str) -> anyhow::Result<String> {
    let bytes = dpapi::unprotect(&from_hex(hex)?)?;
    String::from_utf8(bytes).context("decrypted secret is not valid UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        let bytes = vec![0u8, 1, 15, 16, 255];
        assert_eq!(from_hex(&to_hex(&bytes)).unwrap(), bytes);
        assert!(from_hex("abc").is_err());
        assert!(from_hex("zz").is_err());
    }

    #[cfg(windows)]
    #[test]
    fn dpapi_roundtrip() {
        let secret = "https://discord.com/api/webhooks/123/abc";
        let protected = protect(secret).unwrap();
        assert!(!protected.contains("discord"));
        assert_eq!(unprotect(&protected).unwrap(), secret);
    }
}
