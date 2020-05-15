use smash::{hash40, app, Result, lib::lua_const::*, app::lua_bind::*};
use crate::utils::*;


fn is_landing_lag_param(param_type: u64, param_hash: u64) -> bool{
    if param_hash == 0{
        if [hash40("landing_attack_air_frame_n"), hash40("landing_attack_air_frame_hi"), hash40("landing_attack_air_frame_lw"), hash40("landing_attack_air_frame_f"), hash40("landing_attack_air_frame_b")]
        .contains(&param_type){
            return true;
        }
    }
    return false;
}

/*
-------OPTION A--------- (true)
Upon successful L-Cancel, base landing lag is cut in half, otherwise, return normal landing lag
-------OPTION B--------- (false)
Universally, all base landing lag is multiplied by "universalmul". If you successfully L-Cancel, original landing lag is returned
*/
static option_A_or_B: bool = true;

use crate::L_Cancels::successful_l_cancel;
#[skyline::hook(replace = WorkModule::get_param_float)]
unsafe fn get_param_float_hook(boma: &mut app::BattleObjectModuleAccessor, param_type: u64, param_hash: u64) -> f32{
    
    if get_category(boma) == BATTLE_OBJECT_CATEGORY_FIGHTER && is_landing_lag_param(param_type, param_hash) {
        
        if option_A_or_B { //option A
            if successful_l_cancel[get_player_number(boma)] { 
                return original!()(boma, param_type, param_hash) / 2.;
            }
        }
        else{ //option B
            if !successful_l_cancel[get_player_number(boma)] { 
                return original!()(boma, param_type, param_hash) * 2.;
            }
        }
    
    }

    original!()(boma, param_type, param_hash)
}

pub fn get_param_function_hooks(){
    skyline::install_hook!(get_param_float_hook);
}