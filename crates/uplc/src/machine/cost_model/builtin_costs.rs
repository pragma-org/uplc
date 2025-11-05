mod builtin_costs_v1;
mod builtin_costs_v2;
mod builtin_costs_v3;

use crate::machine::{
    cost_model::{
        builtin_costs::{
            builtin_costs_v1::BuiltinCostsV1, builtin_costs_v2::BuiltinCostsV2,
            builtin_costs_v3::BuiltinCostsV3,
        },
        cost_map::CostMap,
    },
    ExBudget, PlutusVersion,
};

#[derive(Debug, PartialEq)]
pub enum BuiltinCostsVersion {
    BuiltinCostsV1(BuiltinCostsV1),
    BuiltinCostsV2(BuiltinCostsV2),
    BuiltinCostsV3(Box<BuiltinCostsV3>),
}

#[derive(Debug, PartialEq)]
pub struct BuiltinCosts {
    builtin_costs: BuiltinCostsVersion,
}

impl Default for BuiltinCosts {
    fn default() -> Self {
        BuiltinCosts {
            builtin_costs: BuiltinCostsVersion::BuiltinCostsV3(Box::new(BuiltinCostsV3::default())),
        }
    }
}

impl BuiltinCosts {
    pub fn v1() -> Self {
        BuiltinCosts {
            builtin_costs: BuiltinCostsVersion::BuiltinCostsV1(BuiltinCostsV1::default()),
        }
    }
    pub fn v2() -> Self {
        BuiltinCosts {
            builtin_costs: BuiltinCostsVersion::BuiltinCostsV2(BuiltinCostsV2::default()),
        }
    }
    pub fn v3() -> Self {
        BuiltinCosts {
            builtin_costs: BuiltinCostsVersion::BuiltinCostsV3(Box::new(BuiltinCostsV3::default())),
        }
    }

    pub fn initialize_builtin_costs(version: &PlutusVersion, cost_map: &CostMap) -> Self {
        Self {
            builtin_costs: match version {
                PlutusVersion::V1 => BuiltinCostsVersion::BuiltinCostsV1(
                    BuiltinCostsV1::initialize_builtin_costs(cost_map),
                ),
                PlutusVersion::V2 => BuiltinCostsVersion::BuiltinCostsV2(
                    BuiltinCostsV2::initialize_builtin_costs(cost_map),
                ),
                PlutusVersion::V3 => BuiltinCostsVersion::BuiltinCostsV3(Box::new(
                    BuiltinCostsV3::initialize_builtin_costs(cost_map),
                )),
            },
        }
    }

    pub fn get_cost(&self, builtin: &str, args: &[i64]) -> Option<ExBudget> {
        match &self.builtin_costs {
            BuiltinCostsVersion::BuiltinCostsV1(builtin_costs) => {
                builtin_costs.get_cost(builtin, args)
            }
            BuiltinCostsVersion::BuiltinCostsV2(builtin_costs) => {
                builtin_costs.get_cost(builtin, args)
            }
            BuiltinCostsVersion::BuiltinCostsV3(builtin_costs) => {
                builtin_costs.get_cost(builtin, args)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn assert_default_cost_model_v1() {
        let costs = vec![
            100788, 420, 1, 1, 1000, 173, 0, 1, 1000, 59957, 4, 1, 11183, 32, 201305, 8356, 4,
            16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 100, 100,
            16000, 100, 94375, 32, 132994, 32, 61462, 4, 72010, 178, 0, 1, 22151, 32, 91189, 769,
            4, 2, 85848, 228465, 122, 0, 1, 1, 1000, 42921, 4, 2, 24548, 29498, 38, 1, 898148,
            27279, 1, 51775, 558, 1, 39184, 1000, 60594, 1, 141895, 32, 83150, 32, 15299, 32,
            76049, 1, 13169, 4, 22100, 10, 28999, 74, 1, 28999, 74, 1, 43285, 552, 1, 44749, 541,
            1, 33852, 32, 68246, 32, 72362, 32, 7243, 32, 7391, 32, 11546, 32, 85848, 228465, 122,
            0, 1, 1, 90434, 519, 0, 1, 74433, 32, 85848, 228465, 122, 0, 1, 1, 85848, 228465, 122,
            0, 1, 1, 270652, 22588, 4, 1457325, 64566, 4, 20467, 1, 4, 0, 141992, 32, 100788, 420,
            1, 1, 81663, 32, 59498, 32, 20142, 32, 24588, 32, 20744, 32, 25933, 32, 24623, 32,
            53384111, 14333, 10,
        ];

        let cost_model = CostMap::new(&PlutusVersion::V1, &costs);

        assert_eq!(
            BuiltinCosts {
                builtin_costs: BuiltinCostsVersion::BuiltinCostsV1(BuiltinCostsV1::default())
            },
            BuiltinCosts::initialize_builtin_costs(&PlutusVersion::V1, &cost_model)
        );
    }

    #[test]
    fn assert_default_cost_model_v2() {
        let costs = vec![
            100788, 420, 1, 1, 1000, 173, 0, 1, 1000, 59957, 4, 1, 11183, 32, 201305, 8356, 4,
            16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 100, 100,
            16000, 100, 94375, 32, 132994, 32, 61462, 4, 72010, 178, 0, 1, 22151, 32, 91189, 769,
            4, 2, 85848, 228465, 122, 0, 1, 1, 1000, 42921, 4, 2, 24548, 29498, 38, 1, 898148,
            27279, 1, 51775, 558, 1, 39184, 1000, 60594, 1, 141895, 32, 83150, 32, 15299, 32,
            76049, 1, 13169, 4, 22100, 10, 28999, 74, 1, 28999, 74, 1, 43285, 552, 1, 44749, 541,
            1, 33852, 32, 68246, 32, 72362, 32, 7243, 32, 7391, 32, 11546, 32, 85848, 228465, 122,
            0, 1, 1, 90434, 519, 0, 1, 74433, 32, 85848, 228465, 122, 0, 1, 1, 85848, 228465, 122,
            0, 1, 1, 955506, 213312, 0, 2, 270652, 22588, 4, 1457325, 64566, 4, 20467, 1, 4, 0,
            141992, 32, 100788, 420, 1, 1, 81663, 32, 59498, 32, 20142, 32, 24588, 32, 20744, 32,
            25933, 32, 24623, 32, 43053543, 10, 53384111, 14333, 10, 43574283, 26308, 10,
        ];

        let cost_model = CostMap::new(&PlutusVersion::V2, &costs);

        assert_eq!(
            BuiltinCosts {
                builtin_costs: BuiltinCostsVersion::BuiltinCostsV2(BuiltinCostsV2::default())
            },
            BuiltinCosts::initialize_builtin_costs(&PlutusVersion::V2, &cost_model)
        );
    }

    #[test]
    fn assert_default_cost_model_v3() {
        let costs: Vec<i64> = vec![
            100788, 420, 1, 1, 1000, 173, 0, 1, 1000, 59957, 4, 1, 11183, 32, 201305, 8356, 4,
            16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 100, 100,
            16000, 100, 94375, 32, 132994, 32, 61462, 4, 72010, 178, 0, 1, 22151, 32, 91189, 769,
            4, 2, 85848, 123203, 7305, -900, 1716, 549, 57, 85848, 0, 1, 1, 1000, 42921, 4, 2,
            24548, 29498, 38, 1, 898148, 27279, 1, 51775, 558, 1, 39184, 1000, 60594, 1, 141895,
            32, 83150, 32, 15299, 32, 76049, 1, 13169, 4, 22100, 10, 28999, 74, 1, 28999, 74, 1,
            43285, 552, 1, 44749, 541, 1, 33852, 32, 68246, 32, 72362, 32, 7243, 32, 7391, 32,
            11546, 32, 85848, 123203, 7305, -900, 1716, 549, 57, 85848, 0, 1, 90434, 519, 0, 1,
            74433, 32, 85848, 123203, 7305, -900, 1716, 549, 57, 85848, 0, 1, 1, 85848, 123203,
            7305, -900, 1716, 549, 57, 85848, 0, 1, 955506, 213312, 0, 2, 270652, 22588, 4,
            1457325, 64566, 4, 20467, 1, 4, 0, 141992, 32, 100788, 420, 1, 1, 81663, 32, 59498, 32,
            20142, 32, 24588, 32, 20744, 32, 25933, 32, 24623, 32, 43053543, 10, 53384111, 14333,
            10, 43574283, 26308, 10, 16000, 100, 16000, 100, 962335, 18, 2780678, 6, 442008, 1,
            52538055, 3756, 18, 267929, 18, 76433006, 8868, 18, 52948122, 18, 1995836, 36, 3227919,
            12, 901022, 1, 166917843, 4307, 36, 284546, 36, 158221314, 26549, 36, 74698472, 36,
            333849714, 1, 254006273, 72, 2174038, 72, 2261318, 64571, 4, 207616, 8310, 4, 1293828,
            28716, 63, 0, 1, 1006041, 43623, 251, 0, 1, 100181, 726, 719, 0, 1, 100181, 726, 719,
            0, 1, 100181, 726, 719, 0, 1, 107878, 680, 0, 1, 95336, 1, 281145, 18848, 0, 1, 180194,
            159, 1, 1, 158519, 8942, 0, 1, 159378, 8813, 0, 1, 107490, 3298, 1, 106057, 655, 1,
            1964219, 24520, 3,
        ];

        let cost_model = CostMap::new(&PlutusVersion::V3, &costs);

        assert_eq!(
            BuiltinCosts {
                builtin_costs: BuiltinCostsVersion::BuiltinCostsV3(Box::new(
                    BuiltinCostsV3::default()
                ))
            },
            BuiltinCosts::initialize_builtin_costs(&PlutusVersion::V3, &cost_model)
        );
    }
}
