use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

use crate::utils::{random_double, random_double_in};

pub trait Vec3Token {
    type Data: Copy
        + Mul<Output = Self::Data>
        + Div<Output = Self::Data>
        + Add<Output = Self::Data>
        + Sub<Output = Self::Data>;
}

pub struct GeometryToken;
impl Vec3Token for GeometryToken {
    type Data = f64;
}

pub struct Vec3<Token: Vec3Token = GeometryToken>(
    pub Token::Data,
    pub Token::Data,
    pub Token::Data,
);

impl<Token: Vec3Token> Vec3<Token> {
    /// A constructor so users can use type aliases.
    pub const fn new(x: Token::Data, y: Token::Data, z: Token::Data) -> Self {
        Self(x, y, z)
    }
    pub fn length_squared(self) -> Token::Data {
        self.0 * self.0 + self.1 * self.1 + self.2 * self.2
    }

    pub fn dot(self, other: Vec3<Token>) -> Token::Data {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2
    }

    pub fn cross(self, other: Vec3<Token>) -> Vec3<Token> {
        Vec3(
            self.1 * other.2 - self.2 * other.1,
            self.2 * other.0 - self.0 * other.2,
            self.0 * other.1 - self.1 * other.0,
        )
    }
}

impl<Token: Vec3Token<Data = f64>> Vec3<Token> {
    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn unit_vector(self) -> Self {
        self / self.length()
    }

    pub fn random() -> Self {
        Self(random_double(), random_double(), random_double())
    }

    pub fn random_in(min: f64, max: f64) -> Self {
        Self(
            random_double_in(min, max),
            random_double_in(min, max),
            random_double_in(min, max),
        )
    }

    pub fn random_in_unit_sphere() -> Self {
        loop {
            let p = Self::random_in(-1.0, 1.0);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn random_in_unit_disk() -> Self {
        loop {
            let p = Vec3(random_double(), random_double(), 0.0);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn random_unit_vector() -> Self {
        Self::random_in_unit_sphere().unit_vector()
    }

    pub fn random_on_hemisphere(normal: Self) -> Self {
        let on_unit_sphere = Self::random_unit_vector();
        if on_unit_sphere.dot(normal) > 0.0 {
            on_unit_sphere
        } else {
            -on_unit_sphere
        }
    }

    pub fn near_zero(self) -> bool {
        let s = 1e-8;
        self.0.abs() < s && self.1.abs() < s && self.2.abs() < s
    }

    pub fn reflect(self, normal: Self) -> Self {
        self - 2.0 * self.dot(normal) * normal
    }

    pub fn refract(self, normal: Self, etai_over_etat: f64) -> Self {
        let cos_theta = (-self).dot(normal).min(1.0);
        let r_out_perp = etai_over_etat * (self + cos_theta * normal);
        let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * normal;
        r_out_perp + r_out_parallel
    }
}

pub type Point = Vec3<GeometryToken>;

impl<T: Vec3Token> Neg for Vec3<T>
where
    T::Data: Neg<Output = T::Data>,
{
    type Output = Self;
    fn neg(self) -> Vec3<T> {
        Self(-self.0, -self.1, -self.2)
    }
}

macro_rules! impl_binop {
    ($token:tt, $Trait:ident, $fn_name:ident) => {
        impl<T: Vec3Token> $Trait for Vec3<T> where T::Data: $Trait<Output = T::Data> {
            type Output = Self;
            fn $fn_name(self, other: Self) -> Self {
                Self(self.0 $token other.0, self.1 $token other.1, self.2 $token other.2)
            }
        }
    };
}

impl_binop!(+, Add, add);
impl_binop!(-, Sub, sub);
impl_binop!(*, Mul, mul);

macro_rules! impl_binop_assign {
    ($token:tt, $Trait:ident, $fn_name:ident) => {
        impl<T: Vec3Token> $Trait for Vec3<T> where T::Data: $Trait {
            fn $fn_name(&mut self, other: Self) {
                self.0 $token other.0;
                self.1 $token other.1;
                self.2 $token other.2;
            }
        }
    };
}

impl_binop_assign!(+=, AddAssign, add_assign);
impl_binop_assign!(-=, SubAssign, sub_assign);

macro_rules! impl_scalar_op {
    ($token:tt, $Trait:ident, $fn_name:ident) => {
        impl<T: Vec3Token<Data = f64>> $Trait<f64> for Vec3<T> {
            type Output = Self;
            fn $fn_name(self, other: f64) -> Self {
                Self(self.0 $token other, self.1 $token other, self.2 $token other)
            }
        }

        impl<T: Vec3Token<Data = f64>> $Trait<Vec3<T>> for f64 {
            type Output = Vec3<T>;
            fn $fn_name(self, other: Vec3<T>) -> Vec3<T> {
                Vec3(self $token other.0, self $token other.1, self $token other.2)
            }
        }
    };
}

impl_scalar_op!(*, Mul, mul);
impl_scalar_op!(/, Div, div);

impl<T: Vec3Token> Clone for Vec3<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Vec3Token> Copy for Vec3<T> {}
