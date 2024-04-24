use rglua::lua::LuaState;
use rglua::printgm;

pub fn safe_log(state: Option<LuaState>, string: &str) -> () {
    if let Some(l) = state {
        printgm!(l, "{}", string);
    }
}
