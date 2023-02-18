use crate::kernel::kernel_api::KernelInvokeApi;
use crate::{blueprints::transaction_processor::NativeOutput, types::*};
use radix_engine_interface::api::types::{
    NativeInvocation, PackageInvocation,
};

pub fn invoke_native_fn<Y, E>(
    invocation: NativeInvocation,
    api: &mut Y,
) -> Result<Box<dyn NativeOutput>, E>
where
    Y: KernelInvokeApi<E>,
{
    match invocation {
        NativeInvocation::Package(package_invocation) => match package_invocation {
            PackageInvocation::Publish(invocation) => {
                let rtn = api.kernel_invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
    }
}
