use smash::{hash40, app, Result, lib::lua_const::*, app::lua_bind::*};
use crate::utils::*;
use crate::L_Cancels::l_cancel_flag;

const option_A_or_B: &str = "B";

/*  Two types of L-Cancel implementations:

-------OPTION A--------- ("A")
Upon successful L-Cancel, base landing lag is cut in half, otherwise, return normal landing lag... (Melee/PM)

-------OPTION B--------- ("B")
Universally, all base landing lag is multiplied by 2. If you successfully L-Cancel, original landing lag is returned
*/


#[skyline::hook(replace = WorkModule::get_param_float)]
unsafe fn get_param_float_hook(boma: &mut app::BattleObjectModuleAccessor, param_type: u64, param_hash: u64) -> f32{
    
    if get_category(boma) == *BATTLE_OBJECT_CATEGORY_FIGHTER && is_landing_lag_param(param_type, param_hash) {
        
        if option_A_or_B == "A" { //option A
            if l_cancel_flag[get_player_number(boma)] { 
                return original!()(boma, param_type, param_hash) / 2.;
            }
        }
        else if option_A_or_B == "B" { //option B
            if !l_cancel_flag[get_player_number(boma)] { 
                return original!()(boma, param_type, param_hash) * 2.;
            }
        }
    
    }

    original!()(boma, param_type, param_hash)
}



//Checks for if the param passed in is one of the landing lag params
fn is_landing_lag_param(param_type: u64, param_hash: u64) -> bool{
    if param_hash == 0 {
        if [hash40("landing_attack_air_frame_n"), hash40("landing_attack_air_frame_hi"), hash40("landing_attack_air_frame_lw"), hash40("landing_attack_air_frame_f"), hash40("landing_attack_air_frame_b")]
        .contains(&param_type){
            return true;
        }
    }
    return false;
}