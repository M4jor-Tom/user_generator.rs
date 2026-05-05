use rand::Rng;

pub fn modify_email(email: &str, rng: &mut impl Rng) -> String {
    if let Some((local, domain)) = email.split_once('@') {
        if let Some(dot_pos) = domain.rfind('.') {
            let domain_name = &domain[..dot_pos];
            let tld = &domain[dot_pos..];
            let hash: String = (0..4)
                .map(|_| format!("{:x}", rng.gen_range(0..=15)))
                .collect();
            return format!("{}@{}{}{}", local, domain_name, hash, tld);
        }
    }

    email.to_string()
}
