use super::node_modules::type_info::{TypeInfoBlueprint, TypeInfoSubstate};
use crate::errors::RuntimeError;
use crate::errors::SystemUpstreamError;
use crate::kernel::actor::Actor;
use crate::kernel::actor::BlueprintHookActor;
use crate::kernel::actor::FunctionActor;
use crate::kernel::actor::MethodActor;
use crate::kernel::call_frame::Message;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::kernel_api::{KernelApi, KernelInvocation};
use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::system::module::SystemModule;
use crate::system::system::KeyValueEntrySubstate;
use crate::system::system::SystemService;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::SystemModuleMixer;
use crate::track::interface::StoreAccessInfo;
use crate::types::*;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::system_modules::virtualization::OnVirtualizeInput;
use radix_engine_interface::api::system_modules::virtualization::OnVirtualizeOutput;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::account::ACCOUNT_BLUEPRINT;
use radix_engine_interface::blueprints::identity::IDENTITY_BLUEPRINT;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::schema::{InstanceSchema, RefTypes};

#[derive(Clone)]
pub enum SystemLockData {
    KeyValueEntry(KeyValueEntryLockData),
    Field(FieldLockData),
    Default,
}

impl Default for SystemLockData {
    fn default() -> Self {
        SystemLockData::Default
    }
}

#[derive(Clone)]
pub enum KeyValueEntryLockData {
    Read,
    Write {
        schema: ScryptoSchema,
        index: LocalTypeIndex,
        can_own: bool,
    },
    BlueprintWrite {
        blueprint_id: BlueprintId,
        instance_schema: Option<InstanceSchema>,
        type_pointer: TypePointer,
        can_own: bool,
    },
}

#[derive(Clone)]
pub enum FieldLockData {
    Read,
    Write {
        blueprint_id: BlueprintId,
        type_pointer: TypePointer,
    },
}

impl SystemLockData {
    pub fn is_kv_entry(&self) -> bool {
        matches!(self, SystemLockData::KeyValueEntry(..))
    }
}

pub struct SystemConfig<C: SystemCallbackObject> {
    pub callback_obj: C,
    pub blueprint_cache: NonIterMap<CanonicalBlueprintId, BlueprintDefinition>,
    pub schema_cache: NonIterMap<Hash, ScryptoSchema>,
    pub auth_cache: NonIterMap<CanonicalBlueprintId, AuthConfig>,
    pub modules: SystemModuleMixer,
}

impl<C: SystemCallbackObject> KernelCallbackObject for SystemConfig<C> {
    type LockData = SystemLockData;

    fn on_init<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_init(api)
    }

    fn on_teardown<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_teardown(api)
    }

    fn before_drop_node<Y>(node_id: &NodeId, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::before_drop_node(api, node_id)
    }

    fn after_drop_node<Y>(api: &mut Y, total_substate_size: usize) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_drop_node(api, total_substate_size)
    }

    fn before_create_node<Y>(
        node_id: &NodeId,
        node_module_init: &BTreeMap<PartitionNumber, BTreeMap<SubstateKey, IndexedScryptoValue>>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::before_create_node(api, node_id, node_module_init)
    }

    fn before_open_substate<Y>(
        node_id: &NodeId,
        partition_num: &PartitionNumber,
        substate_key: &SubstateKey,
        flags: &LockFlags,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::before_open_substate(api, node_id, partition_num, substate_key, flags)
    }

    fn after_open_substate<Y>(
        handle: LockHandle,
        node_id: &NodeId,
        size: usize,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_open_substate(api, handle, node_id, store_access, size)
    }

    fn on_close_substate<Y>(
        lock_handle: LockHandle,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_close_substate(api, lock_handle, store_access)
    }

    fn on_read_substate<Y>(
        lock_handle: LockHandle,
        value_size: usize,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_read_substate(api, lock_handle, value_size, store_access)
    }

    fn on_write_substate<Y>(
        lock_handle: LockHandle,
        value_size: usize,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_write_substate(api, lock_handle, value_size, store_access)
    }

    fn on_scan_substates<Y>(store_access: &StoreAccessInfo, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_scan_substate(api, store_access)
    }

    fn on_set_substate<Y>(
        value_size: usize,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_set_substate(api, value_size, store_access)
    }

    fn on_take_substates<Y>(store_access: &StoreAccessInfo, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_take_substates(api, store_access)
    }

    fn after_create_node<Y>(
        node_id: &NodeId,
        total_substate_size: usize,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_create_node(api, node_id, total_substate_size, store_access)
    }

    fn before_invoke<Y>(invocation: &KernelInvocation, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::before_invoke(api, invocation)
    }

    fn after_invoke<Y>(output_size: usize, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_invoke(api, output_size)
    }

    fn before_push_frame<Y>(
        callee: &Actor,
        update: &mut Message,
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::before_push_frame(api, callee, update, args)
    }

    fn on_execution_start<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_execution_start(api)
    }

    fn on_execution_finish<Y>(update: &Message, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_execution_finish(api, update)
    }

    fn after_pop_frame<Y>(api: &mut Y, dropped_actor: &Actor) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_pop_frame(api, dropped_actor)
    }

    fn on_allocate_node_id<Y>(entity_type: EntityType, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_allocate_node_id(api, entity_type)
    }

    fn after_move_modules<Y>(
        src_node_id: &NodeId,
        dest_node_id: &NodeId,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_move_modules(api, src_node_id, dest_node_id, store_access)
    }

    //--------------------------------------------------------------------------
    // Note that the following logic doesn't go through mixer and is not costed
    //--------------------------------------------------------------------------

    fn invoke_upstream<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelApi<SystemConfig<C>>,
    {
        let mut system = SystemService::new(api);
        let actor = system.current_actor();
        let receiver = actor
            .try_as_method()
            .map(|x| (x.node_id.clone(), x.is_direct_access));

        // Make dependent resources/components visible
        if let Some(blueprint_id) = actor.blueprint_id() {
            let key = BlueprintVersionKey {
                blueprint: blueprint_id.blueprint_name.clone(),
                version: BlueprintVersion::default(),
            };

            let handle = system.kernel_open_substate_with_default(
                blueprint_id.package_address.as_node_id(),
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_BLUEPRINT_DEPENDENCIES_PARTITION_OFFSET)
                    .unwrap(),
                &SubstateKey::Map(scrypto_encode(&key).unwrap()),
                LockFlags::read_only(),
                Some(|| {
                    let kv_entry = KeyValueEntrySubstate::<()>::default();
                    IndexedScryptoValue::from_typed(&kv_entry)
                }),
                SystemLockData::default(),
            )?;
            system.kernel_read_substate(handle)?;
            system.kernel_close_substate(handle)?;
        }

        match actor {
            Actor::Root => panic!("Root is invoked"),
            Actor::Method(MethodActor {
                module_object_info: ObjectInfo { blueprint_id, .. },
                ident,
                ..
            })
            | Actor::Function(FunctionActor {
                blueprint_id,
                ident,
                ..
            }) => {
                //  Validate input
                let definition = system.get_blueprint_definition(
                    blueprint_id.package_address,
                    &BlueprintVersionKey::new_default(blueprint_id.blueprint_name.as_str()),
                )?;
                let input_type_pointer = definition
                    .interface
                    .get_function_input_type_pointer(ident.as_str())
                    .ok_or_else(|| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::FnNotFound(
                            ident.to_string(),
                        ))
                    })?;
                system.validate_payload_against_blueprint_schema(
                    &blueprint_id,
                    &None,
                    &[(input.as_vec_ref(), input_type_pointer)],
                )?;

                // Validate receiver type
                let function_schema = definition
                    .interface
                    .functions
                    .get(&ident)
                    .expect("Should exist due to schema check");
                match (&function_schema.receiver, receiver) {
                    (Some(receiver_info), Some((_, direct_access))) => {
                        if direct_access
                            != receiver_info.ref_types.contains(RefTypes::DIRECT_ACCESS)
                        {
                            return Err(RuntimeError::SystemUpstreamError(
                                SystemUpstreamError::ReceiverNotMatch(ident.to_string()),
                            ));
                        }
                    }
                    (None, None) => {}
                    _ => {
                        return Err(RuntimeError::SystemUpstreamError(
                            SystemUpstreamError::ReceiverNotMatch(ident.to_string()),
                        ));
                    }
                }

                // Execute
                let export = definition
                    .function_exports
                    .get(&ident)
                    .expect("Schema should have validated this exists")
                    .clone();
                let output =
                    { C::invoke(&blueprint_id.package_address, export, input, &mut system)? };

                // Validate output
                let output_type_pointer = definition
                    .interface
                    .get_function_output_type_pointer(ident.as_str())
                    .expect("Schema verification should enforce that this exists.");
                system.validate_payload_against_blueprint_schema(
                    &blueprint_id,
                    &None,
                    &[(output.as_vec_ref(), output_type_pointer)],
                )?;
                Ok(output)
            }
            Actor::BlueprintHook(BlueprintHookActor { blueprint_id, hook }) => {
                // Find the export
                let definition = system.get_blueprint_definition(
                    blueprint_id.package_address,
                    &BlueprintVersionKey::new_default(blueprint_id.blueprint_name.as_str()),
                )?;
                let export =
                    definition
                        .hook_exports
                        .get(&hook)
                        .ok_or(RuntimeError::SystemUpstreamError(
                            SystemUpstreamError::HookNotFound(hook),
                        ))?;

                // Input is not validated as they're created by system.

                // Invoke the export
                let output = C::invoke(
                    &blueprint_id.package_address,
                    export.clone(),
                    &input,
                    &mut system,
                )?;

                // Check output against well-known schema
                match hook {
                    BlueprintHook::OnVirtualize => {
                        scrypto_decode::<OnVirtualizeOutput>(output.as_slice())
                    }
                    BlueprintHook::OnMove | BlueprintHook::OnDrop | BlueprintHook::OnPersist => {
                        todo!("FIXME add other hook implementations")
                    }
                }
                .map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::OutputDecodeError(e))
                })?;

                Ok(output)
            }
        }
    }

    fn on_substate_lock_fault<Y>(
        node_id: NodeId,
        _partition_num: PartitionNumber,
        _offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        let blueprint_id = match node_id.entity_type() {
            Some(EntityType::GlobalVirtualSecp256k1Account) => {
                BlueprintId::new(&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT)
            }
            Some(EntityType::GlobalVirtualEd25519Account) => {
                BlueprintId::new(&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT)
            }
            Some(EntityType::GlobalVirtualSecp256k1Identity) => {
                BlueprintId::new(&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT)
            }
            Some(EntityType::GlobalVirtualEd25519Identity) => {
                BlueprintId::new(&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT)
            }
            _ => return Ok(false),
        };

        let rtn: Vec<u8> = api
            .kernel_invoke(Box::new(KernelInvocation {
                actor: Actor::blueprint_hook(blueprint_id.clone(), BlueprintHook::OnVirtualize),
                args: IndexedScryptoValue::from_typed(&OnVirtualizeInput { node_id }),
            }))?
            .into();

        let modules: OnVirtualizeOutput =
            scrypto_decode(&rtn).expect("`on_virtualize` output should've been validated");
        let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();
        let address = GlobalAddress::new_or_panic(node_id.into());

        let mut system = SystemService::new(api);
        let address_reservation = system.allocate_virtual_global_address(blueprint_id, address)?;
        system.globalize(modules, Some(address_reservation))?;

        Ok(true)
    }

    fn on_drop_node<Y>(node_id: &NodeId, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        let type_info = TypeInfoBlueprint::get_type(node_id, api)?;
        match type_info {
            TypeInfoSubstate::Object(ObjectInfo { .. }) => {
                todo!()
            }
            TypeInfoSubstate::KeyValueStore(_) => todo!(),
            TypeInfoSubstate::GlobalAddressReservation(_) => todo!(),
            TypeInfoSubstate::GlobalAddressPhantom(_) => todo!(),
        }
    }
}
