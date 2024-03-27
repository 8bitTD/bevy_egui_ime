use bevy_egui::egui;
use bevy::prelude::*;
///////////////////////////////////////// plugin /////////////////////////////////////////
pub struct ImePlugin;

impl Plugin for ImePlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(ImeManager::default())
        .add_systems(PreUpdate,reset_unused_ime)
        .add_systems(Update,listen_ime_events)
        .add_systems(PostUpdate,clear_unused_ime);
    }
}

fn reset_unused_ime(mut ime: ResMut<ImeManager>){//Make all ImeText unused before update
    for i in &mut ime.ime_texts{
        i.is_used = false;
    }
    ime.count = 0;
}

fn listen_ime_events(//ime look
    mut events: EventReader<Ime>,
    mut ime: ResMut<ImeManager>, 
    mut windows: Query<&mut Window>,
) {
    for event in events.read() {
        ime.listen_ime_event(event);
    }
    let mut window = windows.single_mut();
    if window.cursor_position().is_none(){return;}
    window.ime_position = window.cursor_position().unwrap();
}

fn clear_unused_ime(//delete unused ImeText after update
    mut ime: ResMut<ImeManager>, 
) {
    ime.ime_texts.retain(|i| i.is_used == true);
}
//////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Resource)] 
pub struct ImeManager{
    count: usize,
    ime_texts: Vec<ImeText>,
}
impl Default for ImeManager{
    fn default() -> ImeManager{
        ImeManager{
            count: 0,
            ime_texts: Vec::new(),
        }
    }
}
impl ImeManager{
    /// ```
    /// let teo = ime.text_edit_singleline(&mut text, 200.0, ui, ctx);
    /// if teo.response.changed(){
    ///     println!("{:?}", text);
    /// }
    /// ```
    pub fn text_edit_singleline(&mut self, text: &mut String, width: f32, ui: &mut egui::Ui, ctx: &egui::Context) -> egui::text_edit::TextEditOutput{
        if self.count >= self.ime_texts.len(){
            self.add();
            self.ime_texts[self.count].text = text.to_string();
        }
        let teo = self.ime_texts[self.count].get_text_edit_output(width, text, EditType::SingleLine, ui, ctx);
        self.ime_texts[self.count].id = teo.response.id.short_debug_format();
        self.count += 1;
        return teo;
    }
    
    /// ```
    /// let teo = ime.text_edit_multiline(&mut text, 200.0, ui, ctx);
    /// if teo.response.changed(){
    ///     println!("{:?}", text);
    /// }
    /// ```
    pub fn text_edit_multiline(&mut self, text: &mut String, width: f32, ui: &mut egui::Ui, ctx: &egui::Context) -> egui::text_edit::TextEditOutput{
        if self.count >= self.ime_texts.len(){
            self.add();
            self.ime_texts[self.count].text = text.to_string();
        }
        let teo = self.ime_texts[self.count].get_text_edit_output(width, text, EditType::MultiLine, ui, ctx);
        self.ime_texts[self.count].id = teo.response.id.short_debug_format();
        self.count += 1;
        return teo;
    }
    /// ```
    /// let teo = ime.text_edit_multiline(&mut text, 200.0, ui, ctx);
    /// let id = teo.response.id.short_debug_format()
    /// teo.set_text(&id, "あいうえお");
    /// ```
    pub fn set_text(&mut self, id: &str, text: &str){
        let res = self.ime_texts.iter().position(|i|&i.id == id);
        if res.is_none(){return;}
        self.ime_texts[res.unwrap()].text = text.to_string();
    }

    fn add(&mut self){//add Ime
        let it = ImeText::new();
        self.ime_texts.push(it);
    }

    pub fn listen_ime_event(&mut self, event: &Ime){//ime event look
        for i in &mut self.ime_texts{
            i.listen_ime_event(event);
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum EditType{
    SingleLine,
    MultiLine,
}

#[derive(Debug)]
struct ImeText{
    id: String,
    text: String,
    ime_string: String,
    ime_string_index: usize,
    cursor_index: usize,
    is_ime_input: bool,
    is_focus: bool,
    is_ime: bool,
    is_cursor_move: bool,
    edit_type: EditType,
    is_used: bool,
}
impl Default for ImeText{
    fn default() -> Self{
        ImeText{
            id: String::new(),
            text: String::new(),
            ime_string: String::new(),
            ime_string_index: 0,
            cursor_index: 0,
            is_ime_input: false,
            is_focus: false,
            is_ime: false,
            is_cursor_move: true,
            edit_type: EditType::SingleLine,
            is_used: false,
        }
    }
}

impl ImeText{
    fn new() -> ImeText{
        return ImeText::default();
    }

    fn listen_ime_event(&mut self, event: &Ime){
        if !self.is_focus{return;}
        match event {
            Ime::Preedit { value, cursor, .. } if cursor.is_some() => {
                if self.is_focus{ 
                    self.ime_string = value.to_string();
                    self.ime_string_index = self.ime_string.chars().count();
                }
            }
            Ime::Commit { value,.. } => {
                if value.is_empty(){
                    self.is_cursor_move = false;
                }      
                if self.is_focus{
                    let tmp = value.to_string();
                    if self.text.chars().count() == self.cursor_index{
                        self.text.push_str(&tmp);
                    }else{
                        let mut front = String::new();
                        let mut back = String::new();
                        let mut cnt = 0;
                        for c in self.text.chars(){
                            if cnt < self.cursor_index{ front.push_str(&c.to_string()); } 
                            else{ back.push_str(&c.to_string()); }
                            cnt += 1;
                        }                 
                        self.text = format!("{}{}{}", front, tmp, back);
                    }
                    self.is_ime_input = true;
                    self.ime_string = String::new();
                }                
            }
            Ime::Enabled { .. } => { 
                self.is_ime = true;
            }
            Ime::Disabled { .. } => { 
                self.is_ime = false;
            }
            _ => (),
        }
    }

    fn get_text_edit_output(&mut self, width:f32, text: &mut String, edit_type: EditType, ui: &mut egui::Ui, ctx: &egui::Context) -> egui::text_edit::TextEditOutput{
        self.edit_type = edit_type;
        self.is_used = true;
        let mut lyt = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let loj = self.get_layoutjob(string, wrap_width);
            ui.fonts(|f| f.layout_job(loj))
        };
        let mut tmp_text = match self.ime_string.len(){
            0 => {self.text.to_string()},
            _ => {
                let mut front = String::new();
                let mut back = String::new();
                let mut cnt = 0;
                for c in self.text.chars(){
                    if cnt < self.cursor_index{ front.push_str(&c.to_string()); } 
                    else{ back.push_str(&c.to_string()); }
                    cnt += 1;
                }
                format!("{}{}{}", front, self.ime_string, back)
            }
        };

        let mut teo = match self.edit_type{
            EditType::SingleLine => {
                egui::TextEdit::singleline(&mut tmp_text).desired_width(width).layouter(&mut lyt).show(ui)
            },
            _ => {
                egui::TextEdit::multiline(&mut tmp_text).desired_width(width).layouter(&mut lyt).show(ui)
            }
        };
        self.is_focus = teo.response.has_focus();
        if !self.is_ime {self.text = tmp_text.to_string();}
        if teo.cursor_range.is_some(){ 
            self.cursor_index = teo.cursor_range.unwrap().primary.ccursor.index;
        }
        if self.is_ime_input{//respose.changed()=true
            teo.response.mark_changed();
        }
        if self.is_ime_input{ 
            self.is_ime_input = false;
            if self.is_cursor_move{
                let mut res_cursor = teo.cursor_range.unwrap().primary.clone();
                for _ in 0..self.ime_string_index{
                    res_cursor = teo.galley.cursor_right_one_character(&res_cursor);
                }
                let cr = egui::text_selection::CursorRange{
                    primary: res_cursor,
                    secondary: res_cursor,
                };
                teo.state.cursor.set_range(Some(cr));
            }
        }
        if !self.is_cursor_move{
            self.is_cursor_move = true;
        }
        teo.state.clone().store(ctx, teo.response.id);
        *text = self.text.to_string();
        return teo;
    }

    fn get_layoutjob(&self, string: &str, width: f32) -> egui::text::LayoutJob{
        let layout_job = match self.is_ime{
            false => { 
                match self.edit_type{
                    EditType::SingleLine => {
                        egui::text::LayoutJob::simple_singleline(string.into(),egui::FontId::default(), egui::Color32::WHITE) 
                    },
                    _ => {
                        egui::text::LayoutJob::simple(string.into(),egui::FontId::default(), egui::Color32::WHITE, width)
                    },
                }            
            },
            _ => {
                let mut front = String::new();
                let mut back = String::new();
                let mut cnt = 0;
                for c in self.text.chars(){
                    if cnt < self.cursor_index{ front.push_str(&c.to_string()); } 
                    else{ back.push_str(&c.to_string()); }
                    cnt += 1;
                }
    
                let mut lss:Vec<egui::text::LayoutSection> = vec![];
                let mut f_cnt = 0;
                let mut b_cnt = 0;
                b_cnt = b_cnt + front.len();
                let ls_front = egui::text::LayoutSection {
                    leading_space: 0.0,
                    byte_range: f_cnt..b_cnt,
                    format: egui::TextFormat {
                        color: egui::Color32::WHITE,
                        ..Default::default()
                    },
                };
                lss.push(ls_front);
                f_cnt = b_cnt;
    
                b_cnt = b_cnt + self.ime_string.len();
                let ls_text = egui::text::LayoutSection {
                    leading_space: 0.0,
                    byte_range: f_cnt..b_cnt,
                    format: egui::TextFormat {
                        color: egui::Color32::GREEN,
                        background: egui::Color32::from_rgb(0, 128, 64),
                        ..Default::default()
                    },
                };
                lss.push(ls_text);
                f_cnt = b_cnt;
    
                b_cnt = b_cnt + back.len();
                let ls_back = egui::text::LayoutSection {
                    leading_space: 0.0,
                    byte_range: f_cnt..b_cnt,
                    format: egui::TextFormat {
                        color: egui::Color32::WHITE,
                        ..Default::default()
                    },
                };
                lss.push(ls_back);
                let break_on_newline = match self.edit_type{
                    EditType::SingleLine => false,
                    _ => true,
                };
                egui::text::LayoutJob {
                    sections: lss,
                    text: format!("{}{}{}",front, self.ime_string, back),
                    break_on_newline: break_on_newline,  
                    wrap: egui::text::TextWrapping {
                        max_width: width,
                        ..Default::default()
                    },  
                    ..Default::default()
                }
            }
        };
        return layout_job;
    }
}