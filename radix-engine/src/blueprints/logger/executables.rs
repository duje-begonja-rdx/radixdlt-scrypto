use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::blueprints::logger::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::EngineDerefApi;
use radix_engine_interface::api::EngineSubstateApi;

impl ExecutableInvocation for LoggerLogInvocation {
    type Exec = Self;

    fn resolve<D: EngineDerefApi<RuntimeError>>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = ResolvedActor::method(
            NativeFn::Logger(LoggerFn::Log),
            ResolvedReceiver::new(RENodeId::Logger),
        );
        let call_frame_update = CallFrameUpdate::empty();

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for LoggerLogInvocation {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelSubstateApi + EngineSubstateApi<RuntimeError>,
    {
        let offset = SubstateOffset::Logger(LoggerOffset::Logger);
        let node_id = RENodeId::Logger;
        let handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
        let mut substate = api.get_ref_mut(handle)?;
        let logger = substate.logger();
        logger.logs.push((self.level, self.message));

        Ok(((), CallFrameUpdate::empty()))
    }
}
