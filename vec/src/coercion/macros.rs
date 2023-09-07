#[macro_export]
macro_rules! register {
    ( $trait:ident: ($lty:ty , $rty:ty) => $target:ty ) => {
        // register unification into RHS
        impl $trait for ($lty, $rty)
        where
            $lty: super::CoerceInto<$target>,
            $rty: super::CoerceInto<$target>,
        {
            type Common = $target;
            fn as_common(self) -> (Self::Common, Self::Common) {
                (
                    super::CoerceInto::<Self::Common>::coerce(self.0), 
                    super::CoerceInto::<Self::Common>::coerce(self.1)
                )
            }
        }

        // register unification into RHS
        impl $trait for (super::OptionNa<$lty>, $rty)
        where
            $lty: super::CoerceInto<$target>,
            $rty: super::CoerceInto<$target>,
        {
            type Common = $target;
            fn as_common(self) -> (Self::Common, Self::Common) {
                (
                    super::CoerceInto::<Self::Common>::coerce(self.0.inner()),
                    super::CoerceInto::<Self::Common>::coerce(self.1)
                )
            }
        }

        // register unification into RHS
        impl $trait for ($lty, super::OptionNa<$rty>)
        where
            $lty: super::CoerceInto<$target>,
            $rty: super::CoerceInto<$target>,
        {
            type Common = $target;
            fn as_common(self) -> (Self::Common, Self::Common) {
                (
                    super::CoerceInto::<Self::Common>::coerce(self.0), 
                    super::CoerceInto::<Self::Common>::coerce(self.1.inner()),
                )
            }
        }

        // register unification into LHS
        impl $trait for ($rty, $lty)
        where
            $lty: super::CoerceInto<$target>,
            $rty: super::CoerceInto<$target>,
        {
            type Common = $target;
            fn as_common(self) -> (Self::Common, Self::Common) {
                (
                    super::CoerceInto::<Self::Common>::coerce(self.0), 
                    super::CoerceInto::<Self::Common>::coerce(self.1)
                )
            }
        }

        // register unification into LHS
        impl $trait for (super::OptionNa<$rty>, $lty)
        where
            $lty: super::CoerceInto<$target>,
            $rty: super::CoerceInto<$target>,
        {
            type Common = $target;
            fn as_common(self) -> (Self::Common, Self::Common) {
                (
                    super::CoerceInto::<Self::Common>::coerce(self.0.inner()),
                    super::CoerceInto::<Self::Common>::coerce(self.1)
                )
            }
        }

        // register unification into LHS
        impl $trait for ($rty, super::OptionNa<$lty>)
        where
            $lty: super::CoerceInto<$target>,
            $rty: super::CoerceInto<$target>,
        {
            type Common = $target;
            fn as_common(self) -> (Self::Common, Self::Common) {
                (
                    super::CoerceInto::<Self::Common>::coerce(self.0), 
                    super::CoerceInto::<Self::Common>::coerce(self.1.inner()),
                )
            }
        }
    };
}
