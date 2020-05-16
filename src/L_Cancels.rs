use smash::{hash40, app};
use smash::phx::Vector4f;
use smash::lib::{self, L2CAgent, L2CValue, lua_const::*};
use smash::app::{lua_bind::*, sv_animcmd, sv_system};
use skyline::libc::{size_t, c_int, c_void, strlen};
use crate::utils::*;


static mut GLOBAL_FRAME_COUNT: i32 = 0;
static mut curr_frame: i32 = 0;
static mut temp_global_frame: [i32; 8] = [0;8];
static mut lcancelframe: [i32; 8] = [0;8];
pub static mut successful_l_cancel: [bool; 8] = [false;8];
static mut color_flash_flag: [bool; 8] = [false;8];
static cancel_lag: i32 = 15; 



pub unsafe fn l_cancels(boma: &mut app::BattleObjectModuleAccessor, status_kind: i32){
    if status_kind == FIGHTER_STATUS_KIND_ATTACK_AIR{
        if ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_GUARD) || ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_CATCH) {
            successful_l_cancel[get_player_number(boma)] = true;
            lcancelframe[get_player_number(boma)] = MotionModule::frame(boma) as i32;
        }
        if ( MotionModule::frame(boma) as i32 - lcancelframe[get_player_number(boma)] ) > 7 {
            successful_l_cancel[get_player_number(boma)] = false;
            lcancelframe[get_player_number(boma)] = 0;
        }
    }
}

static ground_fix: bool = true; //ground_correct_kind fix for calc's ecb mod/HDR
#[skyline::hook(replace = StatusModule::init_settings)]
pub unsafe fn init_settings_hook(boma: &mut app::BattleObjectModuleAccessor, situation_kind: i32, param_3: i32, param_4: u64, param_5: u64, param_6: bool, param_7: i32, param_8: i32, param_9: i32, param_10: i32){
    let status_kind: i32 = StatusModule::status_kind(boma);
    let mut fix = param_4;
    if get_category(boma) == BATTLE_OBJECT_CATEGORY_FIGHTER {
        //ground_correct_kind fix for calc's ecb mod/HDR
        if ground_fix {
            match fix {
                0 | 3 | 6 | 7 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 30 | 34 | 35 | 126 | 237 => {
                    fix = 1;
                }
                _ => ()
            }
        }

            //variable resets on match start
        if status_kind == FIGHTER_STATUS_KIND_ENTRY {
            GLOBAL_FRAME_COUNT = 0;
            for i in 0..8 {
                temp_global_frame[i] = 0;
                lcancelframe[i] = 0;
                successful_l_cancel[i] = false;
                color_flash_flag[i] = false;
            }
        }

            //L-Cancel-specific variable resets
        if status_kind != FIGHTER_STATUS_KIND_ATTACK_AIR && status_kind != FIGHTER_STATUS_KIND_LANDING_ATTACK_AIR {
            successful_l_cancel[get_player_number(boma)] = false;
            lcancelframe[get_player_number(boma)] = 0;
        }

            //Successful L-Cancel color flash indicator
        if successful_l_cancel[get_player_number(boma)] && status_kind == FIGHTER_STATUS_KIND_LANDING_ATTACK_AIR &&
         !color_flash_flag[get_player_number(boma)] && smash::app::FighterEntryID as i32 != FIGHTER_KIND_NANA {
                temp_global_frame[get_player_number(boma)] = GLOBAL_FRAME_COUNT;
                let colorflashvec1 = Vector4f { /* Red */ x : 1.0, /* Green */ y : 1.0, /* Blue */ z : 1.0, /* Alpha? */ w : 0.1}; // setting this and the next vector's .w to 1 seems to cause a ghostly effect
                let colorflashvec2 = Vector4f { /* Red */ x : 1.0, /* Green */ y : 1.0, /* Blue */ z : 1.0, /* Alpha? */ w : 0.1};
                ColorBlendModule::set_main_color(boma, &colorflashvec1, &colorflashvec2, 0.7, 0.2, 75, true);
                color_flash_flag[get_player_number(boma)] = true;
        }

    }
    original!()(boma, situation_kind, param_3, fix, param_5, param_6, param_7, param_8, param_9, param_10)
}

#[skyline::hook(replace = ControlModule::get_command_flag_cat)]
pub unsafe fn get_command_flag_cat_hook(boma: &mut app::BattleObjectModuleAccessor, category: i32) -> i32{
    if get_category(boma) == BATTLE_OBJECT_CATEGORY_FIGHTER {
        let status_kind: i32 = StatusModule::status_kind(boma);
        if MotionModule::frame(boma) as i32 != curr_frame {
            curr_frame = MotionModule::frame(boma) as i32;
            GLOBAL_FRAME_COUNT += 1;
        }

        if status_kind != FIGHTER_STATUS_KIND_LANDING_ATTACK_AIR && color_flash_flag[get_player_number(boma)] && smash::app::FighterEntryID as i32 != FIGHTER_KIND_NANA {
            ColorBlendModule::cancel_main_color(boma, 0);
            color_flash_flag[get_player_number(boma)] = false;
        }

        l_cancels(boma, status_kind);

    }

    original!()(boma, category)
}

static mut disable_trans_terms: [bool; 8] = [false;8];
#[skyline::hook(replace = WorkModule::is_enable_transition_term)]
pub unsafe fn is_enable_transition_term_hook(boma: &mut app::BattleObjectModuleAccessor, flag: i32) -> bool{

    disable_trans_terms[get_player_number(boma)] = 
        flag == FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_GUARD_ON || flag == FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_CATCH || 
        flag == FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_ESCAPE || flag == FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_ESCAPE_F || flag == FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_ESCAPE_B;

    if disable_trans_terms[get_player_number(boma)] && (GLOBAL_FRAME_COUNT - temp_global_frame[get_player_number(boma)] <= cancel_lag){
        return false;
    }
    
    original!()(boma, flag)
}




pub fn function_hooks(){
    skyline::install_hook!(get_command_flag_cat_hook);
    skyline::install_hook!(init_settings_hook);
    skyline::install_hook!(is_enable_transition_term_hook);
}