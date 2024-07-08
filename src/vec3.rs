use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

pub trait Vec3Token {
    type Data: Copy
        + Mul<Output = Self::Data>
        + Div<Output = Self::Data>
        + Add<Output = Self::Data>
        + Sub<Output = Self::Data>;
}

pub fn vec3(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3(x, y, z)
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
    pub fn new(x: Token::Data, y: Token::Data, z: Token::Data) -> Self {
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

// TODO could introduce num to make this generic
impl<Token: Vec3Token<Data = f64>> Vec3<Token> {
    pub fn length(self) -> Token::Data {
        self.length_squared().sqrt()
    }

    pub fn unit_vector(self) -> Self {
        self / self.length()
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
        impl<T: Vec3Token> $Trait<T::Data> for Vec3<T> {
            type Output = Self;
            fn $fn_name(self, other: T::Data) -> Self {
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