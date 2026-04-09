pub trait Cost<const N: usize> {
    fn cost(&self, args: [i64; N]) -> i64;
}

// Struct using the trait
#[derive(Debug, PartialEq)]
pub struct Costing<const N: usize, T: Cost<N>> {
    pub mem: T,
    pub cpu: T,
}

impl<const N: usize, T> Costing<N, T>
where
    T: Cost<N>,
{
    pub fn new(mem: T, cpu: T) -> Self {
        Self { mem, cpu }
    }
}

#[derive(Debug, PartialEq)]
pub enum OneArgument {
    Constant(i64),
    Linear(LinearSize),
    Quadratic(QuadraticFunction),
}

impl Cost<1> for OneArgument {
    fn cost(&self, args: [i64; 1]) -> i64 {
        let x = args[0];

        match self {
            OneArgument::Constant(c) => *c,
            OneArgument::Linear(m) => m.slope.saturating_mul(x).saturating_add(m.intercept),
            OneArgument::Quadratic(q) => q.coeff_0.saturating_add(q.coeff_1.saturating_mul(x)).saturating_add(q.coeff_2.saturating_mul(x).saturating_mul(x)),
        }
    }
}

pub type OneArgumentCosting = Costing<1, OneArgument>;

impl OneArgumentCosting {
    pub fn constant_cost(c: i64) -> OneArgument {
        OneArgument::Constant(c)
    }

    pub fn linear_cost(intercept: i64, slope: i64) -> OneArgument {
        OneArgument::Linear(LinearSize { intercept, slope })
    }

    pub fn quadratic_cost(coeff_0: i64, coeff_1: i64, coeff_2: i64) -> OneArgument {
        OneArgument::Quadratic(QuadraticFunction {
            coeff_0,
            coeff_1,
            coeff_2,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum TwoArguments {
    ConstantCost(i64),
    LinearInX(LinearSize),
    LinearInY(LinearSize),
    AddedSizes(AddedSizes),
    SubtractedSizes(SubtractedSizes),
    MultipliedSizes(MultipliedSizes),
    MinSize(MinSize),
    MaxSize(MaxSize),
    LinearOnDiagonal(ConstantOrLinear),
    QuadraticInY(QuadraticFunction),
    ConstAboveDiagonalIntoQuadraticXAndY(i64, TwoArgumentsQuadraticFunction),
    ConstAboveDiagonalIntoMultipliedSizes(i64, MultipliedSizes),
    WithInteraction(WithInteraction),
}

pub type TwoArgumentsCosting = Costing<2, TwoArguments>;

impl TwoArgumentsCosting {
    pub fn constant_cost(c: i64) -> TwoArguments {
        TwoArguments::ConstantCost(c)
    }

    pub fn max_size(intercept: i64, slope: i64) -> TwoArguments {
        TwoArguments::MaxSize(MaxSize { intercept, slope })
    }

    pub fn min_size(intercept: i64, slope: i64) -> TwoArguments {
        TwoArguments::MinSize(MinSize { intercept, slope })
    }

    pub fn added_sizes(intercept: i64, slope: i64) -> TwoArguments {
        TwoArguments::AddedSizes(AddedSizes { intercept, slope })
    }

    pub fn multiplied_sizes(intercept: i64, slope: i64) -> TwoArguments {
        TwoArguments::MultipliedSizes(MultipliedSizes { intercept, slope })
    }

    pub fn subtracted_sizes(intercept: i64, slope: i64, minimum: i64) -> TwoArguments {
        TwoArguments::SubtractedSizes(SubtractedSizes {
            intercept,
            slope,
            minimum,
        })
    }

    pub fn linear_on_diagonal(constant: i64, intercept: i64, slope: i64) -> TwoArguments {
        TwoArguments::LinearOnDiagonal(ConstantOrLinear {
            constant,
            intercept,
            slope,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn const_above_diagonal_into_quadratic_x_and_y(
        constant: i64,
        minimum: i64,
        coeff_00: i64,
        coeff_01: i64,
        coeff_02: i64,
        coeff_10: i64,
        coeff_11: i64,
        coeff_20: i64,
    ) -> TwoArguments {
        TwoArguments::ConstAboveDiagonalIntoQuadraticXAndY(
            constant,
            TwoArgumentsQuadraticFunction {
                minimum,
                coeff_00,
                coeff_01,
                coeff_02,
                coeff_10,
                coeff_11,
                coeff_20,
            },
        )
    }

    pub fn linear_in_y(intercept: i64, slope: i64) -> TwoArguments {
        TwoArguments::LinearInY(LinearSize { intercept, slope })
    }

    pub fn linear_in_x(intercept: i64, slope: i64) -> TwoArguments {
        TwoArguments::LinearInX(LinearSize { intercept, slope })
    }

    pub fn quadratic_in_y(coeff_0: i64, coeff_1: i64, coeff_2: i64) -> TwoArguments {
        TwoArguments::QuadraticInY(QuadraticFunction {
            coeff_0,
            coeff_1,
            coeff_2,
        })
    }

    pub fn const_above_diagonal_into_multiplied_sizes(
        constant: i64,
        intercept: i64,
        slope: i64,
    ) -> TwoArguments {
        TwoArguments::ConstAboveDiagonalIntoMultipliedSizes(
            constant,
            MultipliedSizes { intercept, slope },
        )
    }

    pub fn with_interaction(c00: i64, c10: i64, c01: i64, c11: i64) -> TwoArguments {
        TwoArguments::WithInteraction(WithInteraction { c00, c10, c01, c11 })
    }
}

impl Cost<2> for TwoArguments {
    fn cost(&self, args: [i64; 2]) -> i64 {
        let x = args[0];
        let y = args[1];

        match self {
            TwoArguments::ConstantCost(c) => *c,
            TwoArguments::LinearInX(l) => l.slope.saturating_mul(x).saturating_add(l.intercept),
            TwoArguments::LinearInY(l) => l.slope.saturating_mul(y).saturating_add(l.intercept),
            TwoArguments::AddedSizes(s) => s.slope.saturating_mul(x.saturating_add(y)).saturating_add(s.intercept),
            TwoArguments::SubtractedSizes(s) => s.slope.saturating_mul(s.minimum.max(x.saturating_sub(y))).saturating_add(s.intercept),
            TwoArguments::MultipliedSizes(s) => s.slope.saturating_mul(x.saturating_mul(y)).saturating_add(s.intercept),
            TwoArguments::MinSize(s) => s.slope.saturating_mul(x.min(y)).saturating_add(s.intercept),
            TwoArguments::MaxSize(s) => s.slope.saturating_mul(x.max(y)).saturating_add(s.intercept),
            TwoArguments::LinearOnDiagonal(l) => {
                if x == y {
                    x.saturating_mul(l.slope).saturating_add(l.intercept)
                } else {
                    l.constant
                }
            }
            TwoArguments::QuadraticInY(q) => q.coeff_0.saturating_add(q.coeff_1.saturating_mul(y)).saturating_add(q.coeff_2.saturating_mul(y).saturating_mul(y)),
            TwoArguments::ConstAboveDiagonalIntoQuadraticXAndY(constant, q) => {
                if x < y {
                    *constant
                } else {
                    std::cmp::max(
                        q.minimum,
                        q.coeff_00
                            .saturating_add(q.coeff_10.saturating_mul(x))
                            .saturating_add(q.coeff_01.saturating_mul(y))
                            .saturating_add(q.coeff_20.saturating_mul(x).saturating_mul(x))
                            .saturating_add(q.coeff_11.saturating_mul(x).saturating_mul(y))
                            .saturating_add(q.coeff_02.saturating_mul(y).saturating_mul(y)),
                    )
                }
            }
            TwoArguments::ConstAboveDiagonalIntoMultipliedSizes(constant, s) => {
                if x < y {
                    *constant
                } else {
                    s.slope.saturating_mul(x.saturating_mul(y)).saturating_add(s.intercept)
                }
            }
            TwoArguments::WithInteraction(w) => {
                w.c00.saturating_add(w.c10.saturating_mul(x))
                    .saturating_add(w.c01.saturating_mul(y))
                    .saturating_add(w.c11.saturating_mul(x).saturating_mul(y))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ThreeArguments {
    ConstantCost(i64),
    // AddedSizes(AddedSizes),
    LinearInX(LinearSize),
    LinearInY(LinearSize),
    LinearInZ(LinearSize),
    QuadraticInZ(QuadraticFunction),
    LiteralInYorLinearInZ(LinearSize),
    LinearInYAndZ(TwoVariableLinearSize),
    LinearInMaxYZ(LinearSize),
    ExpModCost(ExpModCost),
}

pub type ThreeArgumentsCosting = Costing<3, ThreeArguments>;

impl ThreeArgumentsCosting {
    pub fn constant_cost(c: i64) -> ThreeArguments {
        ThreeArguments::ConstantCost(c)
    }

    pub fn linear_in_z(intercept: i64, slope: i64) -> ThreeArguments {
        ThreeArguments::LinearInZ(LinearSize { intercept, slope })
    }

    pub fn linear_in_y(intercept: i64, slope: i64) -> ThreeArguments {
        ThreeArguments::LinearInY(LinearSize { intercept, slope })
    }

    pub fn literal_in_y_or_linear_in_z(intercept: i64, slope: i64) -> ThreeArguments {
        ThreeArguments::LiteralInYorLinearInZ(LinearSize { intercept, slope })
    }

    pub fn quadratic_in_z(coeff_0: i64, coeff_1: i64, coeff_2: i64) -> ThreeArguments {
        ThreeArguments::QuadraticInZ(QuadraticFunction {
            coeff_0,
            coeff_1,
            coeff_2,
        })
    }

    pub fn linear_in_y_and_z(intercept: i64, slope1: i64, slope2: i64) -> ThreeArguments {
        ThreeArguments::LinearInYAndZ(TwoVariableLinearSize {
            intercept,
            slope1,
            slope2,
        })
    }

    pub fn linear_in_max_y_z(intercept: i64, slope: i64) -> ThreeArguments {
        ThreeArguments::LinearInMaxYZ(LinearSize { intercept, slope })
    }

    pub fn linear_in_x(intercept: i64, slope: i64) -> ThreeArguments {
        ThreeArguments::LinearInX(LinearSize { intercept, slope })
    }

    pub fn exp_mod_cost(coeff_00: i64, coeff_11: i64, coeff_12: i64) -> ThreeArguments {
        ThreeArguments::ExpModCost(ExpModCost {
            coeff_00,
            coeff_11,
            coeff_12,
        })
    }
}

impl Cost<3> for ThreeArguments {
    fn cost(&self, args: [i64; 3]) -> i64 {
        let x = args[0];
        let y = args[1];
        let z = args[2];

        match self {
            ThreeArguments::ConstantCost(c) => *c,
            ThreeArguments::LinearInX(l) => x.saturating_mul(l.slope).saturating_add(l.intercept),
            ThreeArguments::LinearInY(l) => y.saturating_mul(l.slope).saturating_add(l.intercept),
            ThreeArguments::LinearInZ(l) => z.saturating_mul(l.slope).saturating_add(l.intercept),
            ThreeArguments::QuadraticInZ(q) => q.coeff_0.saturating_add(q.coeff_1.saturating_mul(z)).saturating_add(q.coeff_2.saturating_mul(z).saturating_mul(z)),
            ThreeArguments::LiteralInYorLinearInZ(l) => {
                if y == 0 {
                    l.slope.saturating_mul(z).saturating_add(l.intercept)
                } else {
                    y
                }
            }
            ThreeArguments::LinearInYAndZ(l) => y.saturating_mul(l.slope1).saturating_add(z.saturating_mul(l.slope2)).saturating_add(l.intercept),
            ThreeArguments::LinearInMaxYZ(l) => y.max(z).saturating_mul(l.slope).saturating_add(l.intercept),
            ThreeArguments::ExpModCost(c) => {
                let cost = c.coeff_00.saturating_add(c.coeff_11.saturating_mul(y).saturating_mul(z)).saturating_add(c.coeff_12.saturating_mul(y).saturating_mul(z).saturating_mul(z));
                if x <= z {
                    cost
                } else {
                    cost.saturating_add(cost / 2)
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SixArguments {
    ConstantCost(i64),
}

pub type SixArgumentsCosting = Costing<6, SixArguments>;

impl SixArgumentsCosting {
    pub fn constant_cost(c: i64) -> SixArguments {
        SixArguments::ConstantCost(c)
    }
}

impl Cost<6> for SixArguments {
    fn cost(&self, _args: [i64; 6]) -> i64 {
        match self {
            SixArguments::ConstantCost(c) => *c,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LinearSize {
    pub intercept: i64,
    pub slope: i64,
}

#[derive(Debug, PartialEq)]
pub struct TwoVariableLinearSize {
    pub intercept: i64,
    pub slope1: i64,
    pub slope2: i64,
}

#[derive(Debug, PartialEq)]
pub struct AddedSizes {
    pub intercept: i64,
    pub slope: i64,
}

#[derive(Debug, PartialEq)]
pub struct SubtractedSizes {
    pub intercept: i64,
    pub slope: i64,
    pub minimum: i64,
}

#[derive(Debug, PartialEq)]
pub struct MultipliedSizes {
    pub intercept: i64,
    pub slope: i64,
}

#[derive(Debug, PartialEq)]
pub struct MinSize {
    pub intercept: i64,
    pub slope: i64,
}

#[derive(Debug, PartialEq)]
pub struct MaxSize {
    pub intercept: i64,
    pub slope: i64,
}

#[derive(Debug, PartialEq)]
pub struct ConstantOrLinear {
    pub constant: i64,
    pub intercept: i64,
    pub slope: i64,
}

#[derive(Debug, PartialEq)]
pub struct QuadraticFunction {
    coeff_0: i64,
    coeff_1: i64,
    coeff_2: i64,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TwoArgumentsQuadraticFunction {
    minimum: i64,
    coeff_00: i64,
    coeff_01: i64,
    coeff_02: i64,
    coeff_10: i64,
    coeff_11: i64,
    coeff_20: i64,
}

#[derive(Debug, PartialEq)]
pub struct WithInteraction {
    pub c00: i64,
    pub c10: i64,
    pub c01: i64,
    pub c11: i64,
}

#[derive(Debug, PartialEq)]
pub struct ExpModCost {
    coeff_00: i64,
    coeff_11: i64,
    coeff_12: i64,
}
