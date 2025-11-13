#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, Address, symbol_short};

// Structure to track subscription details
#[contracttype]
#[derive(Clone)]
pub struct Subscription {
    pub user: Address,
    pub module_id: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub is_active: bool,
    pub tokens_paid: u64,
}

// Structure to track module information
#[contracttype]
#[derive(Clone)]
pub struct SaaSModule {
    pub module_id: u64,
    pub name: String,
    pub price_per_day: u64,
    pub is_available: bool,
}

// Mapping for storing subscriptions: User Address -> Module ID
#[contracttype]
pub enum SubBook {
    Subscription(Address, u64)
}

// Mapping for storing modules
#[contracttype]
pub enum ModuleBook {
    Module(u64)
}

// Counter for module IDs
const MODULE_COUNT: Symbol = symbol_short!("MOD_CNT");

#[contract]
pub struct MicroSubscriptionContract;

#[contractimpl]
impl MicroSubscriptionContract {

    // Function to create a new SaaS module (Admin function)
    pub fn create_module(env: Env, name: String, price_per_day: u64) -> u64 {
        let mut module_count: u64 = env.storage().instance().get(&MODULE_COUNT).unwrap_or(0);
        module_count += 1;

        let new_module = SaaSModule {
            module_id: module_count,
            name: name.clone(),
            price_per_day,
            is_available: true,
        };

        env.storage().instance().set(&ModuleBook::Module(module_count), &new_module);
        env.storage().instance().set(&MODULE_COUNT, &module_count);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Module created with ID: {}", module_count);
        module_count
    }

    // Function for users to subscribe to a module
    pub fn subscribe(env: Env, user: Address, module_id: u64, duration_days: u64) {
        user.require_auth();

        let module = Self::view_module(env.clone(), module_id);
        
        if !module.is_available {
            log!(&env, "Module is not available");
            panic!("Module not available!");
        }

        let tokens_required = module.price_per_day * duration_days;
        let current_time = env.ledger().timestamp();
        let end_time = current_time + (duration_days * 86400); // 86400 seconds in a day

        let subscription = Subscription {
            user: user.clone(),
            module_id,
            start_time: current_time,
            end_time,
            is_active: true,
            tokens_paid: tokens_required,
        };

        env.storage().instance().set(&SubBook::Subscription(user.clone(), module_id), &subscription);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "User subscribed to module {} for {} days", module_id, duration_days);
    }

    // Function to check subscription status
    pub fn check_subscription(env: Env, user: Address, module_id: u64) -> bool {
        let subscription = Self::view_subscription(env.clone(), user, module_id);
        let current_time = env.ledger().timestamp();

        if subscription.is_active && current_time <= subscription.end_time {
            return true;
        }
        false
    }

    // Function to view module details
    pub fn view_module(env: Env, module_id: u64) -> SaaSModule {
        env.storage().instance().get(&ModuleBook::Module(module_id)).unwrap_or(SaaSModule {
            module_id: 0,
            name: String::from_str(&env, "Not_Found"),
            price_per_day: 0,
            is_available: false,
        })
    }

    // Function to view subscription details
    fn view_subscription(env: Env, user: Address, module_id: u64) -> Subscription {
        env.storage().instance().get(&SubBook::Subscription(user.clone(), module_id)).unwrap_or(Subscription {
            user,
            module_id: 0,
            start_time: 0,
            end_time: 0,
            is_active: false,
            tokens_paid: 0,
        })
    }
}