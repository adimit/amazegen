use crate::{
    generate_seed,
    maze::{
        feature::{Algorithm, Feature, Shape},
        paint::WebColour,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub struct Seed(u64);

impl Default for Seed {
    fn default() -> Self {
        Seed(generate_seed())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WebConfiguration {
    pub seed: Seed,
    pub shape: Shape,
    pub algorithm: Algorithm,
}

pub fn get_default_configuration() -> WebConfiguration {
    WebConfiguration {
        algorithm: Algorithm::GrowingTree,
        seed: Seed(1),
        shape: Shape::Rectilinear(10, 10),
    }
}

impl WebConfiguration {
    pub fn to_configuration(
        &self,
        colour: WebColour,
        features: Vec<Feature>,
        stroke_width: f64,
    ) -> crate::maze::feature::Configuration {
        crate::maze::feature::Configuration {
            seed: self.seed.0,
            shape: self.shape.clone(),
            algorithm: self.algorithm.clone(),
            colour: colour.to_web_string(), // FIXME
            features,
            stroke_width,
        }
    }

    pub fn parse_configuration_from_string(str: String) -> Self {
        let parts = str.split('|').collect::<Vec<_>>();
        let default = get_default_configuration();
        WebConfiguration {
            shape: parts
                .get(0)
                .and_then(|s| {
                    if s.starts_with('S') {
                        s[1..].parse::<usize>().ok().map(Shape::Sigma)
                    } else if s.starts_with('T') {
                        s[1..].parse::<usize>().ok().map(Shape::Theta)
                    } else {
                        s.parse::<usize>()
                            .ok()
                            .map(|size| Shape::Rectilinear(size, size))
                    }
                })
                .unwrap_or(default.shape),
            algorithm: parts
                .get(1)
                .and_then(|s| {
                    if s.eq(&"Kruskal") {
                        Some(Algorithm::Kruskal)
                    } else if s.eq(&"GrowingTree") {
                        Some(Algorithm::GrowingTree)
                    } else {
                        None
                    }
                })
                .unwrap_or(default.algorithm),
            seed: parts
                .get(2)
                .and_then(|s| s.parse::<u64>().ok())
                .map(Seed)
                .unwrap_or(default.seed),
            ..default
        }
    }

    pub fn to_string(&self) -> String {
        let shape = match self.shape {
            Shape::Rectilinear(width, _) => format!("R{}", width),
            Shape::Sigma(size) => format!("S{}", size),
            Shape::Theta(size) => format!("T{}", size),
        };
        let algorithm = match self.algorithm {
            Algorithm::Kruskal => "Kruskal",
            Algorithm::GrowingTree => "GrowingTree",
        };
        format!("{}|{}|{}", shape, algorithm, self.seed.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_configuration_from_string() {
        let config = WebConfiguration::parse_configuration_from_string("".into());
        assert_eq!(config, get_default_configuration());
    }

    #[test]
    fn parses_just_a_seed() {
        let config = WebConfiguration::parse_configuration_from_string("||1234".into());
        let mut default = get_default_configuration().clone();
        default.seed = Seed(1234);
        assert_eq!(config, default);
    }

    #[test]
    fn parses_just_a_size() {
        let config = WebConfiguration::parse_configuration_from_string("12".into());
        assert_eq!(config.shape, Shape::Rectilinear(12, 12));
    }

    #[test]
    fn parses_just_a_shape_spec_hex() {
        let config = WebConfiguration::parse_configuration_from_string("S12".into());
        assert_eq!(config.shape, Shape::Sigma(12));
    }

    #[test]
    fn parses_just_a_shape_spec_circle() {
        let config = WebConfiguration::parse_configuration_from_string("T7".into());
        assert_eq!(config.shape, Shape::Theta(7));
    }

    #[test]
    fn parses_just_an_algorithm() {
        let config = WebConfiguration::parse_configuration_from_string("|Kruskal".into());
        assert_eq!(config.algorithm, Algorithm::Kruskal);
    }

    #[test]
    fn parses_everything_together() {
        let config = WebConfiguration::parse_configuration_from_string("T7|Kruskal|1234".into());
        assert_eq!(config.shape, Shape::Theta(7));
        assert_eq!(config.algorithm, Algorithm::Kruskal);
        assert_eq!(config.seed, Seed(1234));
    }

    #[test]
    fn serialises_configuration_to_hash_string() {
        let config = WebConfiguration {
            shape: Shape::Theta(7),
            algorithm: Algorithm::Kruskal,
            seed: Seed(1234),
            ..get_default_configuration()
        };
        assert_eq!(config.to_string(), "T7|Kruskal|1234");
    }
}
