use smash::app::{self, lua_bind::*, sv_system};
use smash::phx::*;
use smash::hash40;
use smash::lib::{lua_const::*, L2CValue, L2CAgent};
use smash::lua2cpp::L2CFighterCommon;
use crate::utils::*;

pub static mut l_cancel_flag: [bool; 8] = [false;8]; //true if the player has input an l-cancel

static mut aerial_L_press_frame: [i32;8] = [0;8]; //frame(s) since player has input an l-cancel

const L_CANCEL_WINDOW: i32 = 7; //Number of frames before landing to input a shield/grab button that triggers an l-cancel

const L_CANCEL_INPUT_LOCKOUT: f32 = 5.0; //Number of frames during landing to lockout input - helps alleviate shield/grab buffering


#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_status_AttackAir_Main)]
pub unsafe fn status_attackair_hook(fighter: &mut L2CFighterCommon) -> L2CValue {
    //This function runs once per frame during all aerials

    let boma = sv_system::battle_object_module_accessor(fighter.lua_state_agent);

    //if you've hit a shield or grab input, set l_cancel_flag to on.
    if ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_GUARD) || ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_CATCH) {
        l_cancel_flag[get_player_number(boma)] = true;
    }

    //As soon as the l_cancel_flag is on, start counting up every frame to track how long it has been since the l-cancel input
    if l_cancel_flag[get_player_number(boma)] && !StopModule::is_damage(boma) {  // Could add a check for !hitlag to let l-cancel inputs ignore hitlag (I.E. pm/p+)
        aerial_L_press_frame[get_player_number(boma)] += 1;
    }

    //If the l_cancel_flag is still on, and the amount of frames passed since the input is greater than L_CANCEL_WINDOW (7), turn OFF the l_cancel_flag and reset the frame counter... -> in this senario, the player input an l-cancel, but was in the air for more than 7 frames... I.E. mistimed the l-cancel
    if aerial_L_press_frame[get_player_number(boma)] > L_CANCEL_WINDOW && l_cancel_flag[get_player_number(boma)] {
        l_cancel_flag[get_player_number(boma)] = false;
        aerial_L_press_frame[get_player_number(boma)] = 0;
    }

    //call original hooked function
    original!()(fighter)
}


#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_status_LandingAttackAir_Main)]
pub unsafe fn status_landing_attack_air_main_hook(fighter: &mut L2CFighterCommon) -> L2CValue {
    //This function runs once per frame during landings of aerials

    let boma = sv_system::battle_object_module_accessor(fighter.lua_state_agent);

    // if the l_cancel_flag is on, and it's the first frame of whatever animation the player is in,
    //  (in this case, it will - with 100% certainty - be the landing anim)
    // then set the "main color" of the current player to white and reset all l-cancel related flags/timers
    if l_cancel_flag[get_player_number(boma)] && MotionModule::frame(boma) as i32 == 0 && app::utility::get_kind(boma) != *FIGHTER_KIND_NANA {
        let colorflashvec1 = Vector4f { /* Red */ x : 1.0, /* Green */ y : 1.0, /* Blue */ z : 1.0, /* Alpha? */ w : 0.1}; // setting this and the next vector's .w to 1 seems to cause a ghostly effect
        let colorflashvec2 = Vector4f { /* Red */ x : 1.0, /* Green */ y : 1.0, /* Blue */ z : 1.0, /* Alpha? */ w : 0.1};
        ColorBlendModule::set_main_color(boma, &colorflashvec1, &colorflashvec2, 0.7, 0.2, 25, true);
        l_cancel_flag[get_player_number(boma)] = false;
        aerial_L_press_frame[get_player_number(boma)] = 0;
    }

    //If you can interrupt the landing anim (I.E. landing lag is over), remove the white flash
    if CancelModule::is_enable_cancel(boma) {
        ColorBlendModule::cancel_main_color(boma, 0);
    }

    //This just clears the buffer during landings from aerials... helps prevent buffered grab/shield presses
    ControlModule::clear_command(boma, true);

    //call original hooked function
    original!()(fighter)
}


static mut disable_trans_terms: [bool; 8] = [false;8]; //whether the player should be locked out of specified button presses or not

#[skyline::hook(replace = WorkModule::is_enable_transition_term)]
pub unsafe fn is_enable_transition_term_hook(boma: &mut app::BattleObjectModuleAccessor, flag: i32) -> bool{
    // This function returns true/false to determine if the player is "allowed" to go into the status specified by the "flag" param.

    // Sets disable_trans_terms to true/false depending on the current status and currently requested status change.
    // Any transition terms here will be locked out during landing
    disable_trans_terms[get_player_number(boma)] = StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_LANDING_ATTACK_AIR &&
        (flag == FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_GUARD_ON || flag == FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_CATCH || 
        flag == FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_ESCAPE || flag == FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_ESCAPE_F || flag == FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_ESCAPE_B);

    // If the player is inputting one of the statuses to lock out, and your current frame (ignoring motion rates) is less than or equal to L_CANCEL_INPUT_LOCKOUT, return false
    // returning false here essentially means the player WONT transition into the status they are inputting
    // This is done to ensure that players won't accidentally buffer shield or grab during landing because they had input it just before landing
    if disable_trans_terms[get_player_number(boma)] && MotionModule::frame(boma) / MotionModule::rate(boma) as f32 <= L_CANCEL_INPUT_LOCKOUT {
        return false;
    }

    //calls original hooked function
    original!()(boma, flag)
}