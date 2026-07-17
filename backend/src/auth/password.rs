#![allow(dead_code)]

use argon2::{
    password_hash::{
        rand_core::OsRng, Error as PasswordHashError, PasswordHash, PasswordHasher,
        PasswordVerifier, SaltString,
    },
    Argon2,
};

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    validate_password_input(password)?;
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|error| anyhow::anyhow!("failed to hash password: {error}"))?
        .to_string();

    Ok(hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> anyhow::Result<bool> {
    validate_password_input(password)?;
    let parsed_hash = PasswordHash::new(password_hash).map_err(|error| {
        anyhow::anyhow!("stored password hash is not valid PHC format: {error}")
    })?;

    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(PasswordHashError::Password) => Ok(false),
        Err(error) => Err(anyhow::anyhow!("failed to verify password: {error}")),
    }
}

fn validate_password_input(password: &str) -> anyhow::Result<()> {
    if password.is_empty() {
        return Err(anyhow::anyhow!("password must not be empty"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{hash_password, verify_password};

    #[test]
    fn hashes_and_verifies_passwords() {
        let hash = hash_password("correct horse battery staple").expect("hash should be created");

        assert_ne!(hash, "correct horse battery staple");
        assert!(verify_password("correct horse battery staple", &hash).expect("hash should verify"));
        assert!(!verify_password("wrong password", &hash).expect("hash should reject"));
    }

    #[test]
    fn rejects_empty_passwords() {
        assert!(hash_password("").is_err());
        assert!(verify_password("", "$argon2id$v=19$m=1,t=1,p=1$c2FsdA$hash").is_err());
    }
}
