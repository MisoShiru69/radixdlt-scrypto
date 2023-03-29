use super::actor::{Actor, ExecutionMode};
use super::call_frame::{CallFrame, RENodeVisibilityOrigin};
use super::executor::{ExecutableInvocation, Executor, ResolvedInvocation};
use super::heap::{Heap, HeapNode};
use super::id_allocator::IdAllocator;
use super::interpreters::ScryptoInterpreter;
use super::kernel_api::{
    KernelApi, KernelInternalApi, KernelInvokeApi, KernelModuleApi, KernelNodeApi,
    KernelSubstateApi, KernelWasmApi, LockInfo,
};
use super::module::KernelModule;
use super::module_mixer::KernelModuleMixer;
use super::track::{Track, TrackError};
use crate::blueprints::account::AccountSubstate;
use crate::blueprints::identity::IdentityBlueprint;
use crate::blueprints::resource::*;
use crate::errors::RuntimeError;
use crate::errors::*;
use crate::system::kernel_modules::execution_trace::{BucketSnapshot, ProofSnapshot};
use crate::system::node::{ModuleInit, NodeInit};
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::node_properties::VisibilityProperties;
use crate::system::node_substates::{RuntimeSubstate, SubstateRef, SubstateRefMut};
use crate::types::*;
use crate::wasm::WasmEngine;
use native_sdk::modules::access_rules::AccessRulesObject;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::account::{
    ACCOUNT_BLUEPRINT, ACCOUNT_DEPOSIT_BATCH_IDENT, ACCOUNT_DEPOSIT_IDENT,
};
use radix_engine_interface::blueprints::package::PackageCodeSubstate;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::rule;
use radix_engine_interface::schema::KeyValueStoreSchema;
use radix_engine_interface::types::{LockHandle, NodeId, ProofOffset, SubstateId, SubstateKey};
use sbor::rust::mem;

pub struct Kernel<
    'g, // Lifetime of values outliving all frames
    's, // Substate store lifetime
    W,  // WASM engine type
> where
    W: WasmEngine,
{
    /// Current execution mode, specifies permissions into state/invocations
    execution_mode: ExecutionMode,
    /// Stack
    current_frame: CallFrame,
    // This stack could potentially be removed and just use the native stack
    // but keeping this call_frames stack may potentially prove useful if implementing
    // execution pause and/or for better debuggability
    prev_frame_stack: Vec<CallFrame>,
    /// Heap
    heap: Heap,
    /// Store
    track: &'g mut Track<'s>,

    /// ID allocator
    id_allocator: &'g mut IdAllocator,
    /// Interpreter capable of running scrypto programs
    scrypto_interpreter: &'g ScryptoInterpreter<W>,
    /// Kernel module mixer
    module: KernelModuleMixer,
}

impl<'g, 's, W> Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    pub fn new(
        id_allocator: &'g mut IdAllocator,
        track: &'g mut Track<'s>,
        scrypto_interpreter: &'g ScryptoInterpreter<W>,
        module: KernelModuleMixer,
    ) -> Self {
        Self {
            execution_mode: ExecutionMode::Kernel,
            heap: Heap::new(),
            track,
            scrypto_interpreter,
            id_allocator,
            current_frame: CallFrame::new_root(),
            prev_frame_stack: vec![],
            module,
        }
    }

    pub fn initialize(&mut self) -> Result<(), RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::KernelModule, |api| {
            KernelModuleMixer::on_init(api)
        })
    }

    // TODO: Josh holds some concern about this interface; will look into this again.
    pub fn teardown<T>(
        mut self,
        previous_result: Result<T, RuntimeError>,
    ) -> (KernelModuleMixer, Result<T, RuntimeError>) {
        let new_result = match previous_result {
            Ok(output) => {
                // Sanity check call frame
                assert!(self.prev_frame_stack.is_empty());

                // Tear down kernel modules
                match self
                    .execute_in_mode::<_, _, RuntimeError>(ExecutionMode::KernelModule, |api| {
                        KernelModuleMixer::on_teardown(api)
                    }) {
                    Ok(_) => Ok(output),
                    Err(error) => Err(error),
                }
            }
            Err(error) => Err(error),
        };

        (self.module, new_result)
    }

    fn create_virtual_account(
        &mut self,
        global_node_id: NodeId,
        non_fungible_global_id: NonFungibleGlobalId,
    ) -> Result<(), RuntimeError> {
        // TODO: This should move into the appropriate place once virtual manager is implemented
        self.current_frame.add_ref(
            NodeId::GlobalObject(ECDSA_SECP256K1_TOKEN.into()),
            RENodeVisibilityOrigin::Normal,
        );
        self.current_frame.add_ref(
            NodeId::GlobalObject(EDDSA_ED25519_TOKEN.into()),
            RENodeVisibilityOrigin::Normal,
        );

        let access_rule = rule!(require(non_fungible_global_id));
        let component_id = {
            let kv_store_id = {
                let node_id = self.kernel_allocate_node_id(EntityType::InternalKeyValueStore)?;
                let node = NodeInit::KeyValueStore;
                self.kernel_create_node(
                    node_id,
                    node,
                    btreemap!(
                        TypedModuleId::TypeInfo => ModuleInit::TypeInfo(TypeInfoSubstate::KeyValueStore(
                            KeyValueStoreSchema::new::<ResourceAddress, Own>(false))
                        )
                    ),
                )?;
                node_id
            };

            let node_id = {
                let node_modules = btreemap!(
                    TypedModuleId::TypeInfo => ModuleInit::TypeInfo(TypeInfoSubstate::Object {
                        package_address: ACCOUNT_PACKAGE,
                        blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
                        global: false
                    })
                );

                let account_substate = AccountSubstate {
                    vaults: Own::KeyValueStore(kv_store_id.into()),
                };

                let node_id = self.kernel_allocate_node_id(EntityType::Object)?;
                let node = NodeInit::Object(btreemap!(
                    AccountOffset::Account.into() => RuntimeSubstate::Account(account_substate)
                ));
                self.kernel_create_node(node_id, node, node_modules)?;
                node_id
            };
            node_id
        };

        let access_rules = {
            let mut access_rules = AccessRulesConfig::new();
            access_rules.set_access_rule_and_mutability(
                MethodKey::new(
                    TypedModuleId::ObjectState,
                    ACCOUNT_DEPOSIT_IDENT.to_string(),
                ),
                AccessRule::AllowAll,
                AccessRule::DenyAll,
            );
            access_rules.set_access_rule_and_mutability(
                MethodKey::new(
                    TypedModuleId::ObjectState,
                    ACCOUNT_DEPOSIT_BATCH_IDENT.to_string(),
                ),
                AccessRule::AllowAll,
                AccessRule::DenyAll,
            );
            access_rules.default(access_rule.clone(), access_rule)
        };

        let access_rules = AccessRulesObject::sys_new(access_rules, self)?;
        let metadata = Metadata::sys_create(self)?;
        let royalty = ComponentRoyalty::sys_create(RoyaltyConfig::default(), self)?;

        self.globalize_with_address(
            component_id,
            btreemap!(
                TypedModuleId::AccessRules => access_rules.0,
                TypedModuleId::Metadata => metadata.0,
                TypedModuleId::Royalty => royalty.0,
            ),
            global_node_id.into(),
        )?;

        Ok(())
    }

    fn create_virtual_identity(
        &mut self,
        global_node_id: NodeId,
        non_fungible_global_id: NonFungibleGlobalId,
    ) -> Result<(), RuntimeError> {
        // TODO: This should move into the appropriate place once virtual manager is implemented
        self.current_frame.add_ref(
            NodeId::GlobalObject(ECDSA_SECP256K1_TOKEN.into()),
            RENodeVisibilityOrigin::Normal,
        );
        self.current_frame.add_ref(
            NodeId::GlobalObject(EDDSA_ED25519_TOKEN.into()),
            RENodeVisibilityOrigin::Normal,
        );

        let access_rule = rule!(require(non_fungible_global_id));
        let (local_id, access_rules) = IdentityBlueprint::create_virtual(access_rule, self)?;

        let access_rules = AccessRulesObject::sys_new(access_rules, self)?;
        let metadata = Metadata::sys_create(self)?;
        let royalty = ComponentRoyalty::sys_create(RoyaltyConfig::default(), self)?;

        self.globalize_with_address(
            local_id,
            btreemap!(
                TypedModuleId::AccessRules => access_rules.0,
                TypedModuleId::Metadata => metadata.0,
                TypedModuleId::Royalty => royalty.0,
            ),
            global_node_id.into(),
        )?;

        Ok(())
    }

    fn try_virtualize(
        &mut self,
        node_id: NodeId,
        _module_id: TypedModuleId,
        _offset: &SubstateKey,
    ) -> Result<bool, RuntimeError> {
        match node_id {
            // TODO: Need to have a schema check in place before this in order to not create virtual components when accessing illegal substates
            NodeId::GlobalObject(GlobalAddress::Component(component_address)) => {
                // Lazy create component if missing
                match component_address {
                    ComponentAddress::EcdsaSecp256k1VirtualAccount(address) => {
                        self.id_allocator.allocate_virtual_node_id(node_id);
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            ECDSA_SECP256K1_TOKEN,
                            NonFungibleLocalId::bytes(address.to_vec()).unwrap(),
                        );
                        self.create_virtual_account(node_id, non_fungible_global_id)?;
                    }
                    ComponentAddress::EddsaEd25519VirtualAccount(address) => {
                        self.id_allocator.allocate_virtual_node_id(node_id);
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            EDDSA_ED25519_TOKEN,
                            NonFungibleLocalId::bytes(address.to_vec()).unwrap(),
                        );
                        self.create_virtual_account(node_id, non_fungible_global_id)?;
                    }
                    ComponentAddress::EcdsaSecp256k1VirtualIdentity(address) => {
                        self.id_allocator.allocate_virtual_node_id(node_id);
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            ECDSA_SECP256K1_TOKEN,
                            NonFungibleLocalId::bytes(address.to_vec()).unwrap(),
                        );
                        self.create_virtual_identity(node_id, non_fungible_global_id)?;
                    }
                    ComponentAddress::EddsaEd25519VirtualIdentity(address) => {
                        self.id_allocator.allocate_virtual_node_id(node_id);
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            EDDSA_ED25519_TOKEN,
                            NonFungibleLocalId::bytes(address.to_vec()).unwrap(),
                        );
                        self.create_virtual_identity(node_id, non_fungible_global_id)?;
                    }
                    _ => return Ok(false),
                };

                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn drop_node_internal(&mut self, node_id: &NodeId) -> Result<HeapNode, RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::DropNode, |api| match node_id {
            NodeId::Object(..) => api.current_frame.remove_node(&mut api.heap, node_id),
            _ => Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                node_id.clone(),
            ))),
        })
    }

    fn auto_drop_nodes_in_frame(&mut self) -> Result<(), RuntimeError> {
        let owned_nodes = self.current_frame.owned_nodes();
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::AutoDrop, |api| {
            for node_id in owned_nodes {
                if let Ok((package_address, blueprint)) = api.get_object_type_info(node_id) {
                    match (package_address, blueprint.as_str()) {
                        (RESOURCE_MANAGER_PACKAGE, PROOF_BLUEPRINT) => {
                            api.call_function(
                                RESOURCE_MANAGER_PACKAGE,
                                PROOF_BLUEPRINT,
                                PROOF_DROP_IDENT,
                                scrypto_encode(&ProofDropInput {
                                    proof: Proof(node_id.into()),
                                })
                                .unwrap(),
                            )?;
                        }
                        _ => {
                            return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                                node_id,
                            )))
                        }
                    }
                } else {
                    return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                        node_id,
                    )));
                }
            }

            Ok(())
        })?;

        // Last check
        if let Some(node_id) = self.current_frame.owned_nodes().into_iter().next() {
            return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                node_id,
            )));
        }

        Ok(())
    }

    fn run<X: Executor>(
        &mut self,
        mut resolved: Box<ResolvedInvocation<X>>,
    ) -> Result<X::Output, RuntimeError> {
        let caller = Box::new(self.current_frame.actor.clone());

        let executor = &resolved.executor;
        let actor = &resolved.resolved_actor;
        let args = &resolved.args;
        let call_frame_update = &mut resolved.update;

        // Before push call frame
        {
            self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                KernelModuleMixer::before_push_frame(api, &actor, call_frame_update, &args)
            })?;
        }

        // Push call frame
        {
            self.id_allocator.push();

            let frame = CallFrame::new_child_from_parent(
                &mut self.current_frame,
                actor.clone(),
                call_frame_update.clone(),
            )?;
            let parent = mem::replace(&mut self.current_frame, frame);
            self.prev_frame_stack.push(parent);
        }

        // Execute
        let (output, update) = {
            // Handle execution start
            {
                self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                    KernelModuleMixer::on_execution_start(api, &caller)
                })?;
            }

            // Auto drop locks
            self.current_frame
                .drop_all_locks(&mut self.heap, &mut self.track)?;

            // Run
            let (output, mut update) =
                self.execute_in_mode(ExecutionMode::Client, |api| executor.execute(args, api))?;

            // Handle execution finish
            {
                self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                    KernelModuleMixer::on_execution_finish(api, &caller, &mut update)
                })?;
            }

            // Auto-drop locks again in case module forgot to drop
            self.current_frame
                .drop_all_locks(&mut self.heap, &mut self.track)?;

            (output, update)
        };

        // Pop call frame
        {
            let mut parent = self.prev_frame_stack.pop().unwrap();

            // Move resource
            CallFrame::update_upstream(&mut self.current_frame, &mut parent, update)?;

            // drop proofs and check resource leak
            self.auto_drop_nodes_in_frame()?;

            // Restore previous frame
            self.current_frame = parent;

            self.id_allocator.pop()?;
        }

        // After pop call frame
        {
            self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                KernelModuleMixer::after_pop_frame(api)
            })?;
        }

        Ok(output)
    }

    fn verify_valid_mode_transition(
        cur: &ExecutionMode,
        next: &ExecutionMode,
    ) -> Result<(), RuntimeError> {
        match (cur, next) {
            (ExecutionMode::Kernel, ..) => Ok(()),
            (ExecutionMode::Client, ExecutionMode::System) => Ok(()),
            _ => Err(RuntimeError::KernelError(
                KernelError::InvalidModeTransition(*cur, *next),
            )),
        }
    }

    fn invoke_internal<X: Executor>(
        &mut self,
        resolved: Box<ResolvedInvocation<X>>,
    ) -> Result<X::Output, RuntimeError> {
        let depth = self.current_frame.depth;
        // TODO: Move to higher layer
        if depth == 0 {
            for node_id in &resolved.update.node_refs_to_copy {
                match node_id {
                    NodeId::GlobalObject(GlobalAddress::Resource(..)) => {
                        if self.current_frame.get_node_visibility(node_id).is_none() {
                            let offset = TypeInfoOffset::TypeInfo.into();
                            self.track
                                .acquire_lock(
                                    SubstateId(*node_id, TypedModuleId::TypeInfo, offset.clone()),
                                    LockFlags::read_only(),
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                            self.track
                                .release_lock(
                                    SubstateId(*node_id, TypedModuleId::TypeInfo, offset),
                                    false,
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                            self.current_frame
                                .add_ref(*node_id, RENodeVisibilityOrigin::Normal);
                        }
                    }
                    NodeId::GlobalObject(GlobalAddress::Package(package_address)) => {
                        // TODO: Cleanup
                        {
                            if is_native_package(*package_address) {
                                self.current_frame
                                    .add_ref(*node_id, RENodeVisibilityOrigin::Normal);
                                continue;
                            } else {
                                if self.current_frame.get_node_visibility(node_id).is_none() {
                                    let offset = TypeInfoOffset::TypeInfo.into();
                                    self.track
                                        .acquire_lock(
                                            SubstateId(
                                                *node_id,
                                                TypedModuleId::TypeInfo,
                                                offset.clone(),
                                            ),
                                            LockFlags::read_only(),
                                        )
                                        .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                                    self.track
                                        .release_lock(
                                            SubstateId(*node_id, TypedModuleId::TypeInfo, offset),
                                            false,
                                        )
                                        .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                                    self.current_frame
                                        .add_ref(*node_id, RENodeVisibilityOrigin::Normal);
                                }
                            }
                        }
                    }
                    NodeId::GlobalObject(GlobalAddress::Component(global_address)) => {
                        if matches!(
                            global_address,
                            ComponentAddress::EcdsaSecp256k1VirtualAccount(..)
                        ) || matches!(
                            global_address,
                            ComponentAddress::EddsaEd25519VirtualAccount(..)
                        ) || matches!(
                            global_address,
                            ComponentAddress::EcdsaSecp256k1VirtualIdentity(..)
                        ) || matches!(
                            global_address,
                            ComponentAddress::EddsaEd25519VirtualIdentity(..)
                        ) {
                            // For virtual accounts and native packages, create a reference directly
                            self.current_frame
                                .add_ref(*node_id, RENodeVisibilityOrigin::Normal);
                            continue;
                        }

                        if self.current_frame.get_node_visibility(node_id).is_none() {
                            let offset = TypeInfoOffset::TypeInfo.into();
                            self.track
                                .acquire_lock(
                                    SubstateId(*node_id, TypedModuleId::TypeInfo, offset.clone()),
                                    LockFlags::read_only(),
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                            self.track
                                .release_lock(
                                    SubstateId(*node_id, TypedModuleId::TypeInfo, offset),
                                    false,
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                            self.current_frame
                                .add_ref(*node_id, RENodeVisibilityOrigin::Normal);
                        }
                    }
                    NodeId::Object(..) => {
                        if self.current_frame.get_node_visibility(node_id).is_none() {
                            let offset = TypeInfoOffset::TypeInfo.into();
                            self.track
                                .acquire_lock(
                                    SubstateId(*node_id, TypedModuleId::TypeInfo, offset.clone()),
                                    LockFlags::read_only(),
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;

                            let substate_ref =
                                self.track
                                    .get_substate(node_id, TypedModuleId::TypeInfo, &offset);
                            let type_substate: &TypeInfoSubstate = substate_ref.into();
                            if !matches!(type_substate,
                                TypeInfoSubstate::Object {
                                    package_address,
                                    blueprint_name,
                                    ..

                                } if package_address.eq(&RESOURCE_MANAGER_PACKAGE) && blueprint_name.eq(VAULT_BLUEPRINT)
                            ) {
                                return Err(RuntimeError::KernelError(
                                    KernelError::InvalidDirectAccess,
                                ));
                            }

                            self.track
                                .release_lock(
                                    SubstateId(*node_id, TypedModuleId::TypeInfo, offset),
                                    false,
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;

                            self.current_frame
                                .add_ref(*node_id, RENodeVisibilityOrigin::DirectAccess);
                        }
                    }
                    _ => {}
                }
            }
        }

        let output = self.run(resolved)?;

        Ok(output)
    }

    #[inline(always)]
    pub fn execute_in_mode<X, RTN, E>(
        &mut self,
        execution_mode: ExecutionMode,
        execute: X,
    ) -> Result<RTN, RuntimeError>
    where
        RuntimeError: From<E>,
        X: FnOnce(&mut Self) -> Result<RTN, E>,
    {
        Self::verify_valid_mode_transition(&self.execution_mode, &execution_mode)?;

        // Save and replace kernel actor
        let saved = self.execution_mode;
        self.execution_mode = execution_mode;

        let rtn = execute(self)?;

        // Restore old kernel actor
        self.execution_mode = saved;

        Ok(rtn)
    }
}

impl<'g, 's, W> KernelNodeApi for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn kernel_drop_node(&mut self, node_id: &NodeId) -> Result<HeapNode, RuntimeError> {
        KernelModuleMixer::before_drop_node(self, &node_id)?;

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        // TODO: Move this into the system layer
        if let Some(actor) = self.current_frame.actor.clone() {
            let (package_address, blueprint_name) = self.get_object_type_info(node_id.clone())?;
            if !VisibilityProperties::check_drop_node_visibility(
                current_mode,
                &actor,
                package_address,
                blueprint_name.as_str(),
            ) {
                return Err(RuntimeError::KernelError(
                    KernelError::InvalidDropNodeAccess {
                        mode: current_mode,
                        actor: actor.clone(),
                        node_id: node_id.clone(),
                        package_address,
                        blueprint_name,
                    },
                ));
            }
        }

        let node = self.drop_node_internal(node_id)?;

        // Restore current mode
        self.execution_mode = current_mode;

        KernelModuleMixer::after_drop_node(self)?;

        Ok(node)
    }

    fn kernel_allocate_node_id(&mut self, node_type: EntityType) -> Result<NodeId, RuntimeError> {
        // TODO: Add costing
        let node_id = self.id_allocator.allocate_node_id(node_type)?;

        Ok(node_id)
    }

    fn kernel_create_node(
        &mut self,
        node_id: NodeId,
        init: NodeInit,
        module_init: BTreeMap<TypedModuleId, ModuleInit>,
    ) -> Result<(), RuntimeError> {
        KernelModuleMixer::before_create_node(self, &node_id, &init, &module_init)?;

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        match (node_id, &init) {
            (NodeId::GlobalObject(GlobalAddress::Component(..)), NodeInit::GlobalObject(..)) => {}
            (NodeId::GlobalObject(GlobalAddress::Resource(..)), NodeInit::GlobalObject(..)) => {}
            (NodeId::GlobalObject(GlobalAddress::Package(..)), NodeInit::GlobalObject(..)) => {}
            (NodeId::Object(..), NodeInit::Object(..)) => {}
            (NodeId::KeyValueStore(..), NodeInit::KeyValueStore) => {}
            _ => return Err(RuntimeError::KernelError(KernelError::InvalidId(node_id))),
        }

        let push_to_store = match init {
            NodeInit::GlobalObject(..) => true,
            _ => false,
        };

        self.id_allocator.take_node_id(node_id)?;
        self.current_frame.create_node(
            node_id,
            init,
            module_init,
            &mut self.heap,
            &mut self.track,
            push_to_store,
        )?;

        // Restore current mode
        self.execution_mode = current_mode;

        KernelModuleMixer::after_create_node(self, &node_id)?;

        Ok(())
    }
}

impl<'g, 's, W> KernelInternalApi for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn kernel_get_node_visibility_origin(&self, node_id: NodeId) -> Option<RENodeVisibilityOrigin> {
        let visibility = self.current_frame.get_node_visibility(&node_id)?;
        Some(visibility)
    }

    fn kernel_get_module_state(&mut self) -> &mut KernelModuleMixer {
        &mut self.module
    }

    fn kernel_get_current_depth(&self) -> usize {
        self.current_frame.depth
    }

    fn kernel_get_current_actor(&self) -> Option<Actor> {
        self.current_frame.actor.clone()
    }

    fn kernel_read_bucket(&mut self, bucket_id: NodeId) -> Option<BucketSnapshot> {
        if let Ok(substate) = self.heap.get_substate(
            &NodeId::Object(bucket_id),
            TypedModuleId::ObjectState,
            &BucketOffset::Bucket.into(),
        ) {
            let info: &BucketInfoSubstate = substate.into();
            let info = info.clone();

            match info.resource_type {
                ResourceType::Fungible { .. } => {
                    let substate = self
                        .heap
                        .get_substate(
                            &NodeId::Object(bucket_id),
                            TypedModuleId::ObjectState,
                            &BucketOffset::Bucket.into(),
                        )
                        .unwrap();
                    let liquid: &LiquidFungibleResource = substate.into();

                    Some(BucketSnapshot::Fungible {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                        liquid: liquid.amount(),
                    })
                }
                ResourceType::NonFungible { .. } => {
                    let substate = self
                        .heap
                        .get_substate(
                            &NodeId::Object(bucket_id),
                            TypedModuleId::ObjectState,
                            &BucketOffset::Bucket.into(),
                        )
                        .unwrap();
                    let liquid: &LiquidNonFungibleResource = substate.into();

                    Some(BucketSnapshot::NonFungible {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                        liquid: liquid.ids().clone(),
                    })
                }
            }
        } else {
            None
        }
    }

    fn kernel_read_proof(&mut self, proof_id: NodeId) -> Option<ProofSnapshot> {
        if let Ok(substate) = self.heap.get_substate(
            &NodeId::Object(proof_id),
            TypedModuleId::ObjectState,
            &ProofOffset::Proof.into(),
        ) {
            let info: &ProofInfoSubstate = substate.into();
            let info = info.clone();

            match info.resource_type {
                ResourceType::Fungible { .. } => {
                    let substate = self
                        .heap
                        .get_substate(
                            &NodeId::Object(proof_id),
                            TypedModuleId::ObjectState,
                            &ProofOffset::Proof.into(),
                        )
                        .unwrap();
                    let proof: &FungibleProof = substate.into();

                    Some(ProofSnapshot::Fungible {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                        restricted: info.restricted,
                        total_locked: proof.amount(),
                    })
                }
                ResourceType::NonFungible { .. } => {
                    let substate = self
                        .heap
                        .get_substate(
                            &NodeId::Object(proof_id),
                            TypedModuleId::ObjectState,
                            &ProofOffset::Proof.into(),
                        )
                        .unwrap();
                    let proof: &NonFungibleProof = substate.into();

                    Some(ProofSnapshot::NonFungible {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                        restricted: info.restricted,
                        total_locked: proof.non_fungible_local_ids().clone(),
                    })
                }
            }
        } else {
            None
        }
    }
}

impl<'g, 's, W> KernelSubstateApi for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn kernel_lock_substate(
        &mut self,
        node_id: &NodeId,
        module_id: TypedModuleId,
        substate_key: SubstateKey,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        KernelModuleMixer::before_lock_substate(self, &node_id, &module_id, &offset, &flags)?;

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        // TODO: Check if valid offset for node_id

        // Authorization
        if let Some(actor) = &self.current_frame.actor {
            if !VisibilityProperties::check_substate_access(
                current_mode,
                actor,
                node_id,
                offset.clone(),
                flags,
            ) {
                return Err(RuntimeError::KernelError(
                    KernelError::InvalidSubstateAccess {
                        mode: current_mode,
                        actor: actor.clone(),
                        node_id: node_id.clone(),
                        offset,
                        flags,
                    },
                ));
            }
        }

        let maybe_lock_handle = self.current_frame.acquire_lock(
            &mut self.heap,
            &mut self.track,
            node_id,
            module_id,
            offset.clone(),
            flags,
        );

        let lock_handle = match maybe_lock_handle {
            Ok(lock_handle) => lock_handle,
            Err(RuntimeError::KernelError(KernelError::TrackError(TrackError::NotFound(
                SubstateId(node_id, module_id, ref offset),
            )))) => {
                if self.try_virtualize(node_id, module_id, &offset)? {
                    self.current_frame.acquire_lock(
                        &mut self.heap,
                        &mut self.track,
                        &node_id,
                        module_id,
                        offset.clone(),
                        flags,
                    )?
                } else {
                    return maybe_lock_handle;
                }
            }
            Err(err) => {
                match &err {
                    // TODO: This is a hack to allow for package imports to be visible
                    // TODO: Remove this once we are able to get this information through the Blueprint ABI
                    RuntimeError::CallFrameError(CallFrameError::RENodeNotVisible(
                        NodeId::GlobalObject(package_address),
                    )) => {
                        let node_id = NodeId::GlobalObject(*package_address);
                        let module_id = TypedModuleId::ObjectState;
                        self.track
                            .acquire_lock(
                                SubstateId(node_id, module_id, substate_key.clone()),
                                LockFlags::read_only(),
                            )
                            .map_err(|_| err.clone())?;
                        self.track
                            .release_lock(
                                SubstateId(node_id, module_id, substate_key.clone()),
                                false,
                            )
                            .map_err(|_| err)?;
                        self.current_frame
                            .add_ref(node_id, RENodeVisibilityOrigin::Normal);
                        self.current_frame.acquire_lock(
                            &mut self.heap,
                            &mut self.track,
                            &node_id,
                            module_id,
                            offset.clone(),
                            flags,
                        )?
                    }
                    _ => return Err(err),
                }
            }
        };

        // Restore current mode
        self.execution_mode = current_mode;

        // TODO: pass the right size
        KernelModuleMixer::after_lock_substate(self, lock_handle, 0)?;

        Ok(lock_handle)
    }

    fn kernel_get_lock_info(&mut self, lock_handle: LockHandle) -> Result<LockInfo, RuntimeError> {
        self.current_frame.get_lock_info(lock_handle)
    }

    fn kernel_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        KernelModuleMixer::on_drop_lock(self, lock_handle)?;

        self.current_frame
            .drop_lock(&mut self.heap, &mut self.track, lock_handle)?;

        Ok(())
    }

    fn kernel_read_substate(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        // A little hacky: this post sys call is called before the sys call happens due to
        // a mutable borrow conflict for substate ref.
        // Some modules (specifically: ExecutionTraceModule) require that all
        // pre/post callbacks are balanced.
        // TODO: Move post sys call to substate_ref drop() so that it's actually
        // after the sys call processing, not before.

        let substate_ref =
            self.current_frame
                .get_ref(lock_handle, &mut self.heap, &mut self.track)?;
        let ret = substate_ref.to_scrypto_value();

        KernelModuleMixer::on_read_substate(self, lock_handle, ret.as_slice().len())?;

        Ok(ret)
    }

    fn kernel_get_substate_ref<'a, 'b, S>(
        &'b mut self,
        lock_handle: LockHandle,
    ) -> Result<&'a S, RuntimeError>
    where
        &'a S: From<SubstateRef<'a>>,
        'b: 'a,
    {
        KernelModuleMixer::on_read_substate(
            self,
            lock_handle,
            0, //  TODO: pass the right size
        )?;

        let substate_ref =
            self.current_frame
                .get_ref(lock_handle, &mut self.heap, &mut self.track)?;

        Ok(substate_ref.into())
    }

    fn kernel_get_substate_ref_mut<'a, 'b, S>(
        &'b mut self,
        lock_handle: LockHandle,
    ) -> Result<&'a mut S, RuntimeError>
    where
        &'a mut S: From<SubstateRefMut<'a>>,
        'b: 'a,
    {
        // A little hacky: this post sys call is called before the sys call happens due to
        // a mutable borrow conflict for substate ref.
        // Some modules (specifically: ExecutionTraceModule) require that all
        // pre/post callbacks are balanced.
        // TODO: Move post sys call to substate_ref drop() so that it's actually
        // after the sys call processing, not before.
        KernelModuleMixer::on_write_substate(
            self,
            lock_handle,
            0, //  TODO: pass the right size
        )?;

        let substate_ref_mut =
            self.current_frame
                .get_ref_mut(lock_handle, &mut self.heap, &mut self.track)?;

        Ok(substate_ref_mut.into())
    }
}

impl<'g, 's, W> KernelWasmApi<W> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn kernel_create_wasm_instance(
        &mut self,
        package_address: PackageAddress,
        handle: LockHandle,
    ) -> Result<W::WasmInstance, RuntimeError> {
        let substate_ref = self
            .current_frame
            .get_ref(handle, &mut self.heap, &mut self.track)?;
        let code: &PackageCodeSubstate = substate_ref.into();
        let code_size = code.code().len();

        let instance = self
            .scrypto_interpreter
            .create_instance(package_address, &code.code);

        // TODO: move before create_instance() call
        KernelModuleMixer::on_read_substate(self, handle, code_size)?;

        Ok(instance)
    }
}

impl<'g, 's, W, N> KernelInvokeApi<N, RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
    N: ExecutableInvocation,
{
    fn kernel_invoke(
        &mut self,
        invocation: Box<N>,
    ) -> Result<<N as Invocation>::Output, RuntimeError> {
        KernelModuleMixer::before_invoke(
            self,
            &invocation.debug_identifier(),
            invocation.payload_size(),
        )?;

        // Change to kernel mode
        let saved_mode = self.execution_mode;

        self.execution_mode = ExecutionMode::Resolver;
        let resolved = invocation.resolve(self)?;

        self.execution_mode = ExecutionMode::Kernel;
        let rtn = self.invoke_internal(resolved)?;

        // Restore previous mode
        self.execution_mode = saved_mode;

        KernelModuleMixer::after_invoke(
            self, 0, // TODO: Pass the right size
        )?;

        Ok(rtn)
    }
}

impl<'g, 's, W> KernelApi<W, RuntimeError> for Kernel<'g, 's, W> where W: WasmEngine {}

impl<'g, 's, W> KernelModuleApi<RuntimeError> for Kernel<'g, 's, W> where W: WasmEngine {}
