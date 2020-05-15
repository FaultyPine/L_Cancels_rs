use smash::app;
use smash::lib::lua_const::*;

pub fn get_category(boma: &mut app::BattleObjectModuleAccessor) -> i32{
    return (boma.info >> 28) as u8 as i32;
}

pub unsafe fn get_player_number(boma: &mut app::BattleObjectModuleAccessor) -> usize{
    app::lua_bind::WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize
}