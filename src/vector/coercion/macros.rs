#[macro_export]
macro_rules! register {
    ( $trait:ident: ($lty:ty , $rty:ty) => $target:ty ) => {
        // register unification into RHS
        impl $trait for ($lty, $rty)
        where
            $lty: crate::vector::coercion::CoerceInto<$target>,
            $rty: crate::vector::coercion::CoerceInto<$target>,
        {
            type Common = $target;
            fn as_common(self) -> (Self::Common, Self::Common) {
                (
                    crate::vector::coercion::CoerceInto::<Self::Common>::coerce(self.0), 
                    crate::vector::coercion::CoerceInto::<Self::Common>::coerce(self.1)
                )
            }
        }

        // register unification into RHS
        impl $trait for (crate::vector::types::OptionNa<$lty>, $rty)
        where
            $lty: crate::vector::coercion::CoerceInto<$target>,
            $rty: crate::vector::coercion::CoerceInto<$target>,
        {
            type Common = $target;
            fn as_common(self) -> (Self::Common, Self::Common) {
                (
                    crate::vector::coercion::CoerceInto::<Self::Common>::coerce(self.0.inner()),
                    crate::vector::coercion::CoerceInto::<Self::Common>::coerce(self.1)
                )
            }
        }

        // register unification into RHS
        impl $trait for ($lty, crate::vector::types::OptionNa<$rty>)
        where
            $lty: crate::vector::coercion::CoerceInto<$target>,
            $rty: crate::vector::coercion::CoerceInto<$target>,
        {
            type Common = $target;
            fn as_common(self) -> (Self::Common, Self::Common) {
                (
                    crate::vector::coercion::CoerceInto::<Self::Common>::coerce(self.0), 
                    crate::vector::coercion::CoerceInto::<Self::Common>::coerce(self.1.inner()),
                )
            }
        }

        // register unification into LHS
        impl $trait for ($rty, $lty)
        where
            $lty: crate::vector::coercion::CoerceInto<$target>,
            $rty: crate::vector::coercion::CoerceInto<$target>,
        {
            type Common = $target;
            fn as_common(self) -> (Self::Common, Self::Common) {
                (
                    crate::vector::coercion::CoerceInto::<Self::Common>::coerce(self.0), 
                    crate::vector::coercion::CoerceInto::<Self::Common>::coerce(self.1)
                )
            }
        }

        // register unification into LHS
        impl $trait for (crate::vector::types::OptionNa<$rty>, $lty)
        where
            $lty: crate::vector::coercion::CoerceInto<$target>,
            $rty: crate::vector::coercion::CoerceInto<$target>,
        {
            type Common = $target;
            fn as_common(self) -> (Self::Common, Self::Common) {
                (
                    crate::vector::coercion::CoerceInto::<Self::Common>::coerce(self.0.inner()),
                    crate::vector::coercion::CoerceInto::<Self::Common>::coerce(self.1)
                )
            }
        }

        // register unification into LHS
        impl $trait for ($rty, crate::vector::types::OptionNa<$lty>)
        where
            $lty: crate::vector::coercion::CoerceInto<$target>,
            $rty: crate::vector::coercion::CoerceInto<$target>,
        {
            type Common = $target;
            fn as_common(self) -> (Self::Common, Self::Common) {
                (
                    crate::vector::coercion::CoerceInto::<Self::Common>::coerce(self.0), 
                    crate::vector::coercion::CoerceInto::<Self::Common>::coerce(self.1.inner()),
                )
            }
        }
    };
}
