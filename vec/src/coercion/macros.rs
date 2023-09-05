#[macro_export]
macro_rules! register_common_num {
    ( ($lty:ty , $rty:ty) => $target:ty ) => {
        // register unification into RHS
        impl CommonNum for ($lty, $rty)
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
        impl CommonNum for ($rty, $lty)
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
    };
}
