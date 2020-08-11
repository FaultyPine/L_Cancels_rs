use smash::app::{self, lua_bind::*, sv_system};
use smash::phx::*;
use smash::hash40;
use smash::lib::{lua_const::*, L2CValue, L2CAgent};
use smash::lua2cpp::L2CFighterCommon;
use crate::utils::*;

pub static mut l_cancel_flag: [bool; 8] = [false;8];

static mut aerial_L_press_frame: [i32;8] = [0;8];

static L_CANCEL_WINDOW: i32 = 7;

static L_CANCEL_INPUT_LOCKOUT: f32 = 5.0;


//Runs every frame during aerials
#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_status_AttackAir_Main)]
pub unsafe fn status_attackair_hook(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = sv_system::battle_object_module_accessor(fighter.lua_state_agent);
    if ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_GUARD) || ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_CATCH) {
        l_cancel_flag[get_player_number(boma)] = true;
        ControlModule::clear_command(boma, true);
    }
    if l_cancel_flag[get_player_number(boma)] && !StopModule::is_damage(boma) {  // Could add a check for !hitlag to let l-cancel inputs ignore hitlag
        aerial_L_press_frame[get_player_number(boma)] += 1;
        ControlModule::clear_command(boma, true);
    }
    if aerial_L_press_frame[get_player_number(boma)] > L_CANCEL_WINDOW && l_cancel_flag[get_player_number(boma)] {
        l_cancel_flag[get_player_number(boma)] = false;
        aerial_L_press_frame[get_player_number(boma)] = 0;
    }
    original!()(fighter)
}

//Runs every frame during landings of aerials
#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_status_LandingAttackAir_Main)]
pub unsafe fn status_landing_attack_air_main_hook(fighter: &mut L2CFighterCommon) -> L2CValue {
    let boma = sv_system::battle_object_module_accessor(fighter.lua_state_agent);
    if l_cancel_flag[get_player_number(boma)] && MotionModule::frame(boma) as i32 == 0 {
        let colorflashvec1 = Vector4f { /* Red */ x : 1.0, /* Green */ y : 1.0, /* Blue */ z : 1.0, /* Alpha? */ w : 0.1}; // setting this and the next vector's .w to 1 seems to cause a ghostly effect
        let colorflashvec2 = Vector4f { /* Red */ x : 1.0, /* Green */ y : 1.0, /* Blue */ z : 1.0, /* Alpha? */ w : 0.1};
        ColorBlendModule::set_main_color(boma, &colorflashvec1, &colorflashvec2, 0.7, 0.2, 25, true);
        l_cancel_flag[get_player_number(boma)] = false;
        aerial_L_press_frame[get_player_number(boma)] = 0;
    }
    if CancelModule::is_enable_cancel(boma) {
        ColorBlendModule::cancel_main_color(boma, 0);
    }
    ControlModule::clear_command(boma, true);

    original!()(fighter)
}


static mut disable_trans_terms: [bool; 8] = [false;8];
#[skyline::hook(replace = WorkModule::is_enable_transition_term)]
pub unsafe fn is_enable_transition_term_hook(boma: &mut app::BattleObjectModuleAccessor, flag: i32) -> bool{

    disable_trans_terms[get_player_number(boma)] = StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_LANDING_ATTACK_AIR &&
        (flag == FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_GUARD_ON || flag == FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_CATCH || 
        flag == FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_ESCAPE || flag == FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_ESCAPE_F || flag == FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_ESCAPE_B);

    if disable_trans_terms[get_player_number(boma)] && MotionModule::frame(boma) / MotionModule::rate(boma) as f32 <= L_CANCEL_INPUT_LOCKOUT {
        return false;
    }

    original!()(boma, flag)
}