use redis::ToRedisArgs;

macro_rules! impl_prefix {
    ($name:ident, $prefix:expr) => {
        pub struct $name<'a>(pub &'a str);
        impl ToRedisArgs for $name<'_> {
            fn write_redis_args<W>(&self, out: &mut W)
            where
                W: ?Sized + redis::RedisWrite,
            {
                out.write_arg(&[$prefix, self.0.as_bytes()].concat());
            }
        }
    };
}

impl_prefix!(IdOf, b"idof:");
impl_prefix!(Aliases, b"aliases:");
impl_prefix!(Links, b"links:");
