use rglua::interface::Panel;
use rglua::prelude::*;
use crate::hooks::lazy::lazy_detour;
use crate::run_queue;

pub mod lazy;

type PaintTraverseFn = extern "fastcall" fn(&'static Panel, usize, bool, bool);

lazy_detour! {
	pub static LUAL_LOADBUFFERX_H: extern "C" fn(LuaState, *const i8, SizeT, *const i8, *const i8) -> i32 = (
		{
			*LUA_SHARED_RAW.get::<extern "C" fn(LuaState, LuaString, SizeT, LuaString, LuaString) -> i32>(b"luaL_loadbufferx")
				.expect("Failed to get luaL_loadbufferx")
		},
		loadbufferx_h
	);
	pub static PAINT_TRAVERSE_H: PaintTraverseFn = (
		{
			let vgui = iface!(Panel).expect("Failed to get Panel interface");
			std::mem::transmute::<_, PaintTraverseFn>(
				(vgui.vtable as *mut *mut c_void)
					.offset(41)
					.read(),
			)
		},
		paint_traverse_h
	);
}

extern "C" fn loadbufferx_h(
	l: LuaState,
	mut code: LuaString,
	mut code_len: SizeT,
	identifier: LuaString,
	mode: LuaString,
) -> i32 {

	// TODO: Collect all identifiers (filepaths) that are being ran and send them back upstream.
	//  Then send all these file paths back to the web panel and make a file editor that allows you to read any client
	//  file, then modify and re-run it LIVE within the game.

	unsafe { LUAL_LOADBUFFERX_H.call(l, code, code_len, identifier, mode) }
}

extern "fastcall" fn paint_traverse_h(
	this: &'static Panel,
	panel_id: usize,
	force_repaint: bool,
	force_allow: bool,
) {
	unsafe {
		PAINT_TRAVERSE_H.call(this, panel_id, force_repaint, force_allow);
	}

	// Run operations queue.
	run_queue(None);
}

#[derive(Debug, thiserror::Error)]
pub enum HookingError {
	#[error("Failed to hook function: {0}")]
	Detour(#[from] detour::Error),

	#[error("Failed to get interface")]
	Interface(#[from] rglua::interface::Error),
}

pub fn attach() -> Result<(), HookingError> {
	use once_cell::sync::Lazy;

	Lazy::force(&LUAL_LOADBUFFERX_H);
	Lazy::force(&PAINT_TRAVERSE_H);

	Ok(())
}

pub fn detach() -> Result<(), detour::Error> {
	unsafe {
		LUAL_LOADBUFFERX_H.disable()?;
		PAINT_TRAVERSE_H.disable()?;
	}
	Ok(())
}