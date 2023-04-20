#![no_std]
extern crate alloc;

use aidoku::{
	error::Result,
	prelude::*,
	std::{json, String, Vec},
	Chapter, Filter, FilterType, Listing, Manga, MangaPageResult, Page,
};
use alloc::string::ToString;

mod crypto;
mod helper;
mod parser;

const FILTER_THEME: [&str; 61] = [
	"",
	"aiqing",
	"huanlexiang",
	"maoxian",
	"qihuan",
	"baihe",
	"xiaoyuan",
	"kehuan",
	"dongfang",
	"danmei",
	"shenghuo",
	"gedou",
	"qingxiaoshuo",
	"xuanyi",
	"qita",
	"shengui",
	"zhichang",
	"mengxi",
	"teenslove",
	"zhiyu",
	"changtiao",
	"sige",
	"jiecao",
	"jianniang",
	"jingji",
	"gaoxiao",
	"weiniang",
	"rexue",
	"lizhi",
	"hougong",
	"meishi",
	"xingzhuanhuan",
	"zhentan",
	"COLOR",
	"aa",
	"yinyuewudao",
	"yishijie",
	"zhanzheng",
	"lishi",
	"jingsong",
	"jizhan",
	"mohuan",
	"dushi",
	"chuanyue",
	"kongbu",
	"comiket100",
	"chongsheng",
	"comiket99",
	"comiket97",
	"comiket101",
	"comiket96",
	"zhaixi",
	"wuxia",
	"shengcun",
	"C98",
	"comiket95",
	"fate",
	"zhuansheng",
	"Uncensored",
	"xianxia",
	"loveLive",
];
const FILTER_ORDERING: [&str; 2] = ["popular", "datetime_updated"];

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query = String::new();
	let mut theme = String::new();
	let mut ordering = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query = filter.value.as_string()?.read();
			}
			FilterType::Select => {
				let index = filter.value.as_int()? as usize;
				match filter.name.as_str() {
					"题材" => {
						theme = FILTER_THEME[index].to_string();
					}
					_ => continue,
				}
			}
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int()? as usize;
				let ascending = value.get("ascending").as_bool().unwrap_or(false);
				ordering.push_str(if ascending { "" } else { "-" });
				ordering.push_str(FILTER_ORDERING[index]);
			}
			_ => continue,
		}
	}

	let url = if query.is_empty() {
		helper::gen_explore_url(theme, ordering, page)
	} else {
		helper::gen_search_url(query, page)
	};
	let json = helper::get_json(url);
	let data = json.get("results").as_object()?;
	let list = data.get("list").as_array()?;

	Ok(MangaPageResult {
		manga: parser::parse_manga_list(list),
		has_more: parser::has_more(data),
	})
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let mut date_type = String::new();
	let mut is_recs = false;
	let mut is_newest = false;

	match listing.name.as_str() {
		"日榜" => {
			date_type.push_str("day");
		}
		"周榜" => {
			date_type.push_str("week");
		}
		"月榜" => {
			date_type.push_str("month");
		}
		"总榜" => {
			date_type.push_str("total");
		}
		"编辑推荐" => {
			is_recs = true;
		}
		"全新上架" => {
			is_newest = true;
		}
		_ => return get_manga_list(Vec::new(), page),
	}

	let url = if !date_type.is_empty() {
		helper::gen_rank_url(date_type, page)
	} else if is_recs {
		helper::gen_recs_url(page)
	} else if is_newest {
		helper::gen_newest_url(page)
	} else {
		String::new()
	};

	let json = helper::get_json(url);
	let data = json.get("results").as_object()?;
	let list = data.get("list").as_array()?;

	Ok(MangaPageResult {
		manga: parser::parse_manga_list(list),
		has_more: parser::has_more(data),
	})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = helper::gen_manga_details_url(id);
	let json = helper::get_json(url);
	let data = json.get("results").as_object()?;

	Ok(parser::parse_manga(data))
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = helper::gen_chapter_list_url(id);
	let json = helper::get_json(url);
	let data = json.get("results").as_string()?.read();
	let data = helper::decrypt(data);
	let data = json::parse(data)?.as_object()?;

	Ok(parser::parse_chapter_list(data))
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = helper::gen_page_list_url(manga_id, chapter_id);
	let json = helper::get_json(url);
	let data = json.get("results").as_object()?;
	let data = data.get("chapter").as_object()?;

	Ok(parser::parse_page_list(data))
}
