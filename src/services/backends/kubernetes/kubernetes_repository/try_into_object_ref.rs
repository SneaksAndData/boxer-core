use kube::runtime::reflector::{Lookup, ObjectRef};
use regex::Regex;

pub trait TryIntoObjectRef<R>
where
    R: Lookup,
    R::DynamicType: Default,
{
    type Error;

    fn try_into_object_ref(self, namespace: String) -> Result<ObjectRef<R>, Self::Error>;
}

impl<R> TryIntoObjectRef<R> for String
where
    R: Lookup,
    R::DynamicType: Default,
{
    type Error = anyhow::Error;

    fn try_into_object_ref(self, namespace: String) -> Result<ObjectRef<R>, Self::Error> {
        let only_dns_subdomain = Regex::new(r"[^-a-z0-9]")?;
        let lowercase_name = self.to_lowercase();
        let safe_name = only_dns_subdomain.replace_all(&lowercase_name, "-").to_string();
        let trimmed_name = safe_name.trim_matches('-');
        let mut or = ObjectRef::new(&trimmed_name);
        or.namespace = Some(namespace);
        Ok(or)
    }
}

impl<R> TryIntoObjectRef<R> for (String, String)
where
    R: Lookup,
    R::DynamicType: Default,
{
    type Error = anyhow::Error;

    fn try_into_object_ref(self, namespace: String) -> Result<ObjectRef<R>, Self::Error> {
        format!("{}-{}", self.0, self.1)
            .to_string()
            .try_into_object_ref(namespace)
    }
}
