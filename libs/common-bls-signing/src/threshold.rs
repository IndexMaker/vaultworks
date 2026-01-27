use bls12_381::{G1Projective, G2Projective, Scalar};
use common_bls::affine::{PublicKey, Signature};
use ff::Field;
use pairing::group::Curve;
use rand_core::RngCore;

pub struct KeySet {
    pub master_secret: Scalar,
    pub participant_shares: Vec<(u64, Scalar)>, // (ID, PrivateKey)
}

/// Generate Master Private Key and Participant Private Keys
///
/// This will generate random polynomial, where value at the point x = 0 is a Master Secret,
/// and then values at next points x=1..N are Participant Shares of the secret.
///
/// This way Participant Shares are point on the polynomial of degree M = (threshold_count - 1).
/// We get N = total_count of Participant Shares, and to reconstruct the polynomial we only need M points.
///
pub fn generate_threshold_keys(
    threshold_count: usize,
    total_count: usize,
    mut rng: impl RngCore,
) -> KeySet {
    // Generate Master Secret f(x = 0) = a_0
    let master_secret = Scalar::random(&mut rng);

    // Generate random coefficients a_1, a_2, ..., a_{M-1}
    let mut coefficients = vec![master_secret];
    for _ in 1..threshold_count {
        coefficients.push(Scalar::random(&mut rng));
    }

    // Generate Participant Shares as points on polynomial at x = 1, 2, ..., N
    let mut participant_shares = Vec::new();
    for id in 1..=total_count {
        let x = Scalar::from(id as u64);
        let mut y = Scalar::zero();

        // Compute: f(x) = a_0 + a_1 * x^1 + a_2 x^2 + ...
        let mut x_pow = Scalar::one();
        for coeff in &coefficients {
            y += (*coeff) * x_pow;
            x_pow *= x;
        }

        participant_shares.push((id as u64, y));
    }

    KeySet {
        master_secret,
        participant_shares,
    }
}

/// Reconstruct the Master Public Key from a subset of M public key shares
///
/// We only need to do that for a subset of M public keys, because we know that
/// each Private Key is a point on the polynomial of M - 1 degree, and we only
/// need M points to reconstruct that polynomial.
///
pub fn recover_master_public_key(shares: &[(u64, PublicKey)]) -> Result<PublicKey, Vec<u8>> {
    let mut acc = G1Projective::identity();

    for (i, (id_i, pk_i)) in shares.iter().enumerate() {
        let xi = Scalar::from(*id_i);

        // Compute Lagrange Coefficient for this share at x = 0:
        //
        //      lambda_i = product( x_j / (x_j - x_i) ) for all j != i
        //
        let mut numerator = Scalar::one();
        let mut denominator = Scalar::one();

        for (j, (id_j, _)) in shares.iter().enumerate() {
            if i == j {
                continue;
            }
            let xj = Scalar::from(*id_j);
            numerator *= xj;
            denominator *= xj - xi;
        }

        let inverse = denominator
            .invert()
            .into_option()
            .ok_or_else(|| b"Failed to invert denominator")?;

        let lambda_i = numerator * inverse;
        let weighted_share = G1Projective::from(*pk_i) * lambda_i;

        acc += weighted_share;
    }

    // Convert back to Affine for the final Master Public Key
    Ok(acc.to_affine())
}

/// Combine multiple signatures into single signature
///
///
pub fn combine_signatures(shares: &[(u64, Signature)]) -> Result<Signature, Vec<u8>> {
    let mut combined = G2Projective::identity();

    let xs: Vec<Scalar> = shares.iter().map(|(id, _)| Scalar::from(*id)).collect();

    for (i, (_, sig)) in shares.iter().enumerate() {
        // Calculate Lagrange Coefficient lambda_i (for this participant) at point x = 0:
        //
        //      lambda_i = product(-x_i / (x_i - x_j)) for all j != i
        //
        let mut lambda = Scalar::one();
        let xi = xs[i];

        for (j, xj) in xs.iter().enumerate() {
            if i == j {
                continue;
            }

            let numerator = -(*xj);
            let denominator = xi - xj;

            let inverse = denominator
                .invert()
                .into_option()
                .ok_or_else(|| b"Failed to invert denominator")?;

            lambda *= numerator * inverse;
        }

        // 3. Scale signature share: sig_i * lambda_i
        let mut scaled_sig = G2Projective::from(sig);
        scaled_sig *= lambda;

        // 4. Accumulate
        combined += scaled_sig;
    }

    Ok(combined.to_affine())
}
