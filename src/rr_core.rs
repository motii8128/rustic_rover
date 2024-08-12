mod dualshock;
mod interface;
mod thread_connection;
mod packet;
mod utils;
mod serial;
mod save_data;

use dualshock::DualShock4DriverManager;
use interface::{AppState, ControllerConnectionType, DualShock4, Packet, RRMessage, LifeCycle};

use iced::{self, Element};
use iced::widget::{button, column, combo_box, row, text};
use save_data::SaveDataManager;
use serial::SerialManager;
use utils::path_to_image;

pub struct RusticRover
{
    game_controller_manager:dualshock::DualShock4DriverManager,
    controller_connection_types_combo_box:utils::ComboBox<ControllerConnectionType>,
    packet_creator:packet::PacketCreator,
    controller_state:AppState,
    serial_state:AppState,
    packet_state:AppState,
    life_cycle:LifeCycle,
    serial_manager:serial::SerialManager,
    input_path:String,
    sd_manager:save_data::SaveDataManager,
    selected_file_name:String,
}

impl iced::Application for RusticRover {
    type Executor = iced::executor::Default;
    type Message = interface::RRMessage;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let app = RusticRover
        {
            game_controller_manager:DualShock4DriverManager::new(),
            controller_connection_types_combo_box:utils::ComboBox::new(ControllerConnectionType::ALL.to_vec()),
            packet_creator:packet::PacketCreator::new(),
            controller_state:AppState::NoReady,
            serial_state:AppState::NoReady,
            packet_state:AppState::NoReady,
            life_cycle:LifeCycle::Setting,
            serial_manager:SerialManager::new(),
            input_path:String::new(),
            sd_manager:SaveDataManager::new(),
            selected_file_name:String::new()
        };

        (app, iced::Command::none())
    }

    fn title(&self) -> String {
        String::from("RusticRover")
    }

    fn theme(&self) -> Self::Theme {
        iced::Theme::Light
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::subscription::unfold(
            "subscribe_controller_input", 
            self.game_controller_manager.first_connector.subscriber.take(), 
            move |mut subscriber|async move{
                let get = subscriber.as_mut().unwrap().recv().await.unwrap();

                (interface::RRMessage::ControllerThreadMessage(get), subscriber)
            })
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            interface::RRMessage::ControllerThreadMessage(ds4)=>{
                self.game_controller_manager.get_value[0] = ds4;
                for i in 1..self.game_controller_manager.controller_num
                {
                    self.game_controller_manager.get_value[i] = self.game_controller_manager.connectors[i].subscriber.recv().unwrap();
                }
                
                self.packet_creator.create_packet(ds4);

                match self.packet_creator.packet_ {
                    Some(p)=>{
                        self.packet_state = AppState::OK;
                        
                        if self.serial_state == AppState::OK
                        {
                            self.serial_manager.conn.publisher.send(p).unwrap();
                        }
                    }
                    None=>{
                        self.packet_state = AppState::NoReady;
                    }
                }
            }
            interface::RRMessage::ControllerType(type_)=>{
                self.controller_connection_types_combo_box.selected = Some(type_);
            }
            interface::RRMessage::ControllerStart=>{
                if self.controller_connection_types_combo_box.selected == None
                {
                    self.controller_state = AppState::NoReady;
                }
                else 
                {
                    self.game_controller_manager.scan_device();
                    if !self.game_controller_manager.device_list.is_empty()
                    {
                        match self.controller_connection_types_combo_box.selected {
                            Some(type_)=>{
                                self.game_controller_manager.spawn_driver(type_);
                                self.game_controller_manager.controller_num += 1;
                                self.controller_state = AppState::OK;
                                self.life_cycle = LifeCycle::Home;
                            }
                            None=>{
                                self.controller_state = AppState::ERROR;
                            }
                        }
                    }

                    
                    self.sd_manager.search_data_files();
                }
            }
            interface::RRMessage::AddController=>{
                if self.game_controller_manager.controller_num < 3
                {
                    if !self.game_controller_manager.device_list.is_empty()
                    {
                        match self.controller_connection_types_combo_box.selected {
                            Some(type_)=>{
                                let new_connector = thread_connection::ThreadConnector::<DualShock4>::new();
                                self.game_controller_manager.connectors.push(new_connector);
                                let index = self.game_controller_manager.controller_num;
                                self.game_controller_manager.add_driver(
                                    type_, 
                                    self.game_controller_manager.connectors.get(index).unwrap().publisher.clone());

                                self.game_controller_manager.controller_num += 1;
                                self.game_controller_manager.get_value.push(DualShock4::new());
                                self.controller_state = AppState::OK;
                            }
                            None=>{
                                self.controller_state = AppState::ERROR;
                            }
                        }
                    }
                    else {
                        self.controller_state = AppState::ERROR;
                    }
                }
            }
            interface::RRMessage::PowerRateX(get_rate)=>{
                self.packet_creator.x_pow_rate = get_rate
            }
            interface::RRMessage::PowerRateY(get_rate)=>{
                self.packet_creator.y_pow_rate = get_rate
            }
            interface::RRMessage::PowerRateRotation(get_rate)=>{
                self.packet_creator.ro_pow_rate = get_rate;
            }
            interface::RRMessage::PowerRateM1(get_rate)=>{
                self.packet_creator.m1_pow_rate = get_rate;
            }
            interface::RRMessage::PowerRateM2(get_rate)=>{
                self.packet_creator.m2_pow_rate = get_rate;
            }
            interface::RRMessage::PacketAssign1p(a1p)=>{
                self.packet_creator.x_cb.plus.selected = Some(a1p)
            }
            interface::RRMessage::PacketAssign1m(a1m)=>{
                self.packet_creator.x_cb.minus.selected = Some(a1m)
            }
            interface::RRMessage::PacketAssign2p(a2p)=>{
                self.packet_creator.y_cb.plus.selected = Some(a2p)
            }
            interface::RRMessage::PacketAssign2m(a2m)=>{
                self.packet_creator.y_cb.minus.selected = Some(a2m)
            }
            interface::RRMessage::PacketAssign3p(a3p)=>{
                self.packet_creator.ro_cb.plus.selected = Some(a3p)
            }
            interface::RRMessage::PacketAssign3m(a3m)=>{
                self.packet_creator.ro_cb.minus.selected = Some(a3m)
            }
            interface::RRMessage::PacketAssign4p(a4p)=>{
                self.packet_creator.m1_cb.plus.selected = Some(a4p)
            }
            interface::RRMessage::PacketAssign4m(a4m)=>{
                self.packet_creator.m1_cb.minus.selected = Some(a4m)
            }
            interface::RRMessage::PacketAssign5p(a5p)=>{
                self.packet_creator.m2_cb.plus.selected = Some(a5p)
            }
            interface::RRMessage::PacketAssign5m(a5m)=>{
                self.packet_creator.m2_cb.minus.selected = Some(a5m)
            },
            interface::RRMessage::SerialSearch=>{
                self.serial_manager.search_port();
            }
            interface::RRMessage::SerialStart=>{
                match self.serial_manager.path_list {
                    Some(_)=>{
                        let con_p = thread_connection::ThreadConnector::<Packet>::new();
                        self.serial_manager.conn.publisher = con_p.publisher.clone();
                        let port_name_ = self.input_path.clone();

                        self.serial_state = AppState::OK;
                
                        std::thread::spawn(move || serial::serial_task(port_name_, con_p.subscriber));
                    }
                    None=>{

                    }
                }    
            }
            interface::RRMessage::PortList(port_name)=>{
                self.input_path = port_name;
            }
            interface::RRMessage::FileSelect(selected)=>{
                self.selected_file_name = selected;

                self.sd_manager.load_from_file(self.selected_file_name.clone());

                self.packet_creator.x_cb.plus.selected = self.sd_manager.xp_assign;
                self.packet_creator.x_cb.minus.selected = self.sd_manager.xm_assign;
                self.packet_creator.x_pow_rate = self.sd_manager.x_rate.unwrap();
                self.packet_creator.y_cb.plus.selected = self.sd_manager.yp_assign;
                self.packet_creator.y_cb.minus.selected = self.sd_manager.ym_assign;
                self.packet_creator.y_pow_rate = self.sd_manager.y_rate.unwrap();
                self.packet_creator.ro_cb.plus.selected = self.sd_manager.rop_assign;
                self.packet_creator.ro_cb.minus.selected = self.sd_manager.rom_assign;
                self.packet_creator.ro_pow_rate = self.sd_manager.ro_rate.unwrap();
                self.packet_creator.m1_cb.plus.selected = self.sd_manager.m1p_assign;
                self.packet_creator.m1_cb.minus.selected = self.sd_manager.m1m_assign;
                self.packet_creator.m1_pow_rate = self.sd_manager.m1_rate.unwrap();
                self.packet_creator.m2_cb.plus.selected = self.sd_manager.m2p_assign;
                self.packet_creator.m2_cb.minus.selected = self.sd_manager.m2m_assign;
                self.packet_creator.m2_pow_rate = self.sd_manager.m2_rate.unwrap();
            }
            interface::RRMessage::CycleHome=>{
                self.life_cycle = LifeCycle::Home;
            }
            interface::RRMessage::CycleController=>{
                self.life_cycle = LifeCycle::ControllerInfo
            }
            interface::RRMessage::CyclePacket=>{
                self.life_cycle = LifeCycle::PacketInfo
            }
            interface::RRMessage::CycleSerial=>{
                self.life_cycle = LifeCycle::SerialInfo
            }
        }

        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        if self.life_cycle == LifeCycle::Setting
        {
            self.title_view()
        }
        else if self.life_cycle == LifeCycle::Home
        {
            let icon = path_to_image("./rustic_rover.png", 600);
            
            column![icon,self.home_view()].align_items(iced::Alignment::Center).align_items(iced::alignment::Horizontal::Center.into()).padding(10).into()
        }
        else if self.life_cycle == LifeCycle::ControllerInfo
        {
            let add_con = utils::normal_size_button("AddController", RRMessage::AddController);
            let combo_ = combo_box(
                &self.controller_connection_types_combo_box.all, 
                "Select Controller Connection Method", 
                self.controller_connection_types_combo_box.selected.as_ref(), 
            interface::RRMessage::ControllerType);
            column![self.controller_view(), self.home_view(), combo_, add_con].align_items(iced::Alignment::Center).padding(10).spacing(50).into()
        }
        else if self.life_cycle == LifeCycle::PacketInfo
        {
            let f_v = self.sd_manager.menu_view(self.selected_file_name.clone());
            
            column![self.packet_creator.packet_view(), f_v, self.home_view()].align_items(iced::Alignment::Center).padding(10).spacing(50).into()
        }
        else if self.life_cycle == LifeCycle::SerialInfo
        {
            column![self.serial_view(), self.home_view()].align_items(iced::alignment::Horizontal::Center.into()).padding(10).spacing(50).into()
        }
        else {
            text("LifeCycleError!!").size(300).into()
        }
    }
}

impl RusticRover {
    fn title_view(&self)->Element<'_, interface::RRMessage, iced::Theme, iced::Renderer>
    {
        let title = text("RusticRover").size(200).horizontal_alignment(iced::alignment::Horizontal::Center);
        let combo_ = combo_box(
            &self.controller_connection_types_combo_box.all, 
            "Select Controller Connection Method", 
            self.controller_connection_types_combo_box.selected.as_ref(), 
        interface::RRMessage::ControllerType);

        let path = "./rustic_rover.png";

        let img = utils::path_to_image(path, 1000);

        let btn = button("Start").on_press(interface::RRMessage::ControllerStart).width(iced::Length::Shrink).height(iced::Length::Shrink);

        let err_text = utils::setting_state_logger(self.controller_state);

        column![title, combo_, btn, err_text,img].align_items(iced::alignment::Alignment::Center).padding(10).spacing(50).into()

    }
    fn home_view(&self)->Element<'_, interface::RRMessage, iced::Theme, iced::Renderer>
    {
        let home_btn = utils::normal_size_button("Home", RRMessage::CycleHome).width(100);
        
        let con_btn = utils::normal_size_button("ControllerInfo", RRMessage::CycleController);
        let con_state = utils::state_to_image(self.controller_state);
        let con_clm = column![con_btn, con_state].align_items(iced::Alignment::Center);

        let serial_btn = utils::normal_size_button("SerialInfo", RRMessage::CycleSerial);
        let serial_state = utils::state_to_image(self.serial_state);
        let serial_clm = column![serial_btn, serial_state].align_items(iced::Alignment::Center);

        let packet_btn = utils::normal_size_button("PacketInfo", RRMessage::CyclePacket);
        let packet_state = utils::state_to_image(self.packet_state);
        let packet_clm = column![packet_btn, packet_state].align_items(iced::Alignment::Center);

        row![home_btn, con_clm, packet_clm, serial_clm].spacing(50).padding(10).align_items(iced::Alignment::End).into()
    }
    fn controller_view(&self)->Element<'_, interface::RRMessage, iced::Theme, iced::Renderer>
    {
        let tit = text("Controller Info").size(100);
        
        match self.game_controller_manager.controller_num {
            1=>{
                let con_1 = input_to_controller_view(self.game_controller_manager.get_value[0]);
                column![tit, con_1].padding(10).into()
            }
            2=>{
                let con_1 = input_to_controller_view(self.game_controller_manager.get_value[0]);
                let con_2 = input_to_controller_view(self.game_controller_manager.get_value[1]);

                column![tit, con_1, con_2].padding(10).into()
            }
            3=>{
                let con_1 = input_to_controller_view(self.game_controller_manager.get_value[0]);
                let con_2 = input_to_controller_view(self.game_controller_manager.get_value[1]);
                let con_3 = input_to_controller_view(self.game_controller_manager.get_value[2]);

                column![tit, con_1, con_2, con_3].padding(10).into()
            }
            _=>{
                text("GameControllerManager Error!!").size(300).into()
            }
        }
    }
    fn serial_view(&self)->Element<'_, interface::RRMessage, iced::Theme, iced::Renderer>
    {
        match &self.serial_manager.path_list {
            Some(get_list)=>{
                let combo_yp = combo_box(
                    &get_list.all, 
                    "Select Serial Port", 
                    Some(&self.input_path), 
                    RRMessage::PortList);
                let start_b = utils::normal_size_button("Start Serial", RRMessage::SerialStart);
                let b = utils::normal_size_button("Rescan Serial", RRMessage::SerialSearch);

                column![b, combo_yp, start_b].align_items(iced::alignment::Alignment::Center).padding(10).spacing(50).into()
            }
            None=>{
                let serial_text = text("Press Button and search serialport").size(30);
                let b = utils::normal_size_button("Scan Serial", RRMessage::SerialSearch);
                column![serial_text, b].align_items(iced::alignment::Alignment::Center).padding(10).spacing(50).into()
            }
        }
    }
}

fn input_to_controller_view<'a>(input:DualShock4)->Element<'a, interface::RRMessage, iced::Theme, iced::Renderer>
{
    let con_state = if input.state
            {
                "Connected!!"
            }
            else
            {
                "Not Connected"
            };
            let state_tex = text(format!("Type:{}\nState:{}\n",input.mode, con_state)).size(25);
            let joy_tex = text(format!("JoyStick\nleft_x:{:2.5}\nleft_y:{:2.5}\nright_x:{:2.5}\nright_y:{:2.5}", 
                input.sticks.left_x,
                input.sticks.left_y,
                input.sticks.right_x,
                input.sticks.right_y)).size(25);
            let dpad_tex = text(format!("DPad\nup:{:5}\ndown:{:5}\nright:{:5}\nleft:{:5}", 
                input.dpad.up_key,
                input.dpad.down_key,
                input.dpad.right_key,
                input.dpad.left_key)).size(25);
            let btn_tex = text(format!("Buttons\ncircle:{:5},cross:{:5}\ncube:{:5},triangle:{:5}\nR1:{},R2:{}\nL1:{},L2:{}", 
                input.btns.circle,input.btns.cross,
                input.btns.cube,input.btns.triangle,
                input.btns.r1,input.btns.r2,
                input.btns.l1,input.btns.l2)).size(25);

            row![state_tex, joy_tex, dpad_tex, btn_tex].padding(10).spacing(30).into()
}