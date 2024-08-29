pub mod serial;
pub mod udp;
pub mod interface;

use interface::Packet;
use crate::rr_core::interface::{SerialMessage, RRMessage};
use crate::rr_core::thread_connection::{ThreadConnector, ThreadManager};
use crate::rr_core::utils::{ComboBox, LogManager};

use iced_aw::TabLabel;

pub struct SerialManager
{
    pub driver_num:usize,
    pub id:Vec<usize>,
    pub id_box:ComboBox<usize>,
    pub is_small_packet:bool,
    pub is_smooth:bool,
    pub conn:Vec<ThreadConnector<Packet>>,
    pub thread_manager:Vec<ThreadManager>,
    pub path_list:Option<ComboBox<String>>,
    pub selected:String,
    pub smooth_value:i32,
    pub logger:LogManager
}

impl SerialManager {
    pub fn view(&self)->iced::Element<'_, RRMessage>
    {
        use iced::widget::{button, column, text, container::Container};
        match &self.path_list {
            Some(get_list)=>{
                let p_config_text = text("Packet Config").size(80);
                use iced::widget::checkbox;
                use iced_aw::number_input;
                
                let is_sp = checkbox("Use Small Packet", self.is_small_packet).on_toggle(SerialMessage::SetPacketSize);
                let is_smooth = checkbox("Use Smooth", self.is_smooth).on_toggle(SerialMessage::SetSmooth);

                let sm_gain_item = if self.is_smooth
                {
                    Some(number_input(self.smooth_value, 20, SerialMessage::SmoothValue).step(1))
                }
                else
                {
                    None
                };

                let packet_config_clm = match sm_gain_item {
                    Some(sm_gain)=>{
                        iced::widget::column![p_config_text, is_sp, is_smooth, sm_gain].spacing(30)
                    }
                    None=>{
                        iced::widget::column![p_config_text, is_sp, is_smooth].spacing(30)
                    }
                };
                


                let port_config_text = text("Port Config").size(80);
                use iced::widget::combo_box;
                let combo_yp = combo_box(
                    &get_list.all, 
                    "Select Serial Port", 
                    Some(&self.selected), 
                    SerialMessage::PortSelected);
                
                let start_b = button("Start Serial").width(iced::Length::Shrink).height(iced::Length::Shrink).on_press(SerialMessage::SerialStart);
                let scan_b = button("Scan Port").width(iced::Length::Shrink).height(iced::Length::Shrink).on_press(SerialMessage::SerialScan);

                let port_config_clm = iced::widget::column![port_config_text, scan_b, combo_yp, start_b].spacing(30);

                let id_config_text = text("Thread Config").size(80);
                let id_combo_box = combo_box(
                    &self.id_box.all, 
                    "Select id that you want to stop", 
                    self.id_box.selected.as_ref(), 
                    SerialMessage::ThreadID
                );
                let stop = button("Stop Button").width(iced::Length::Shrink).height(iced::Length::Shrink).on_press(SerialMessage::ThreadStop);

                let id_config_clm = iced::widget::column![id_config_text, id_combo_box, stop].spacing(30);

                use iced::widget::row;
                let above_row = row![packet_config_clm, port_config_clm].spacing(400);

                let state_log = self.logger.view().size(50);
                let container:iced::Element<'_, SerialMessage> = Container::new(
                    column![above_row, id_config_clm, state_log].align_items(iced::Alignment::Center).padding(10).spacing(50)
                )
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center).into();

                container.map(RRMessage::Serial)
            }
            None=>{
                let serial_text = text("Press Button and search serialport").size(100);
                let b = button("Scan Port").width(iced::Length::Shrink).height(iced::Length::Shrink).on_press(SerialMessage::SerialScan);
                
                let container:iced::Element<'_, SerialMessage> = Container::new(
                    column![serial_text, b].align_items(iced::Alignment::Center).padding(10).spacing(50)
                )
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center).into();

                container.map(RRMessage::Serial)
            }
        }
    }
    pub fn update(&mut self, message:SerialMessage)
    {
        match message {
            SerialMessage::PortSelected(name)=>{
                self.selected = name.clone();
                self.logger.add_str(format!("Port path selected: {}", name));
            }
            SerialMessage::SerialScan=>{
                self.search_port();
                self.logger.add_str(format!("Search available port."));
            }
            SerialMessage::SerialStart=>{
                if self.is_smooth
                {
                    self.spawn_smooth_serial(self.smooth_value);
                    self.logger.add_str(format!("Start Serial with smoother at {}", self.selected.clone()));
                }
                else {
                    self.spawn_serial();
                    self.logger.add_str(format!("Start Serial at {}", self.selected.clone()));
                }
            }
            SerialMessage::SetPacketSize(changed)=>{
                self.is_small_packet = changed;

                if changed
                {
                    self.logger.add_str(format!("Set small packet is enable."));
                }
                else {
                    self.logger.add_str(format!("Set small packet is disable."));
                }
            }
            SerialMessage::ThreadID(id)=>{
                self.id_box.selected = Some(id);

                self.logger.add_str(format!("Select thread id :{}", id));
            }
            SerialMessage::ThreadStop=>{
                match self.id_box.selected {
                    Some(id_)=>{
                        self.thread_manager[id_].thread_stop();
                        self.conn.remove(id_);
                        self.driver_num -= 1;
                        self.id.remove(id_);

                        self.logger.add_str(format!("Stop the thread. ID is {}", id_));
                    }
                    None=>{
                        self.logger.add_str(format!("Don't select thread ID."));
                    }
                }
            }
            SerialMessage::SmoothValue(val)=>{
                self.smooth_value = val;

                self.logger.add_str(format!("Set smooth gain : {}", val));
            }
            SerialMessage::SetSmooth(sm)=>{
                self.is_smooth = sm;

                if sm
                {
                    self.logger.add_str(format!("Set smoother to enable"));
                }
                else {
                    self.logger.add_str(format!("Set smoother to disable"));
                }
            }
        }
    }
    fn title(&self)->String
    {
        String::from("シリアル設定")
    }
    pub fn tab_label(&self)->TabLabel
    {
        TabLabel::Text(self.title())
    }
}

impl SerialManager {
    pub fn new()->SerialManager
    {
        let mut v = Vec::<ThreadConnector<Packet>>::new();
        v.push(ThreadConnector::<Packet>::new());
        let mut manager_vec = Vec::<ThreadManager>::new();
        manager_vec.push(ThreadManager::new());
        let id_v = Vec::<usize>::new();
        SerialManager {
            driver_num:0, 
            is_small_packet:false,
            conn: v, 
            path_list : None, 
            selected:String::new(), 
            thread_manager:manager_vec, 
            id:id_v.clone(), 
            id_box:ComboBox::<usize>::new(id_v.clone()), 
            smooth_value:1, 
            is_smooth:false,
            logger:LogManager::new()
        }
    }
    pub fn search_port(&mut self)
    {
        match serialport::available_ports()
        {
            Ok(vec)=>{
                let mut path_list_ = Vec::<String>::new();

                for i in 0..vec.len()
                {
                    if !vec.get(i).unwrap().port_name.contains("/dev/ttyS")
                    {
                        path_list_.push(vec.get(i).unwrap().port_name.clone())
                    }
                }

                self.path_list = Some(ComboBox::new(path_list_));
            }
            Err(_e)=>{
                self.path_list = None
            }
        }
    }
    pub fn spawn_serial(&mut self)
    {
        let selected_port = self.selected.clone();
        let node = ThreadConnector::<Packet>::new();
        self.conn[self.driver_num].publisher = node.publisher.clone();
        let clone_ = self.thread_manager[self.driver_num].get_clone();
        self.id.push(self.driver_num);
        self.id_box = ComboBox::new(self.id.clone());

        self.driver_num += 1;
        self.thread_manager.push(ThreadManager::new());
        self.conn.push(ThreadConnector::<Packet>::new());
        let is_ = self.is_small_packet.clone();

        std::thread::spawn(move ||{
            let mut port_ = serialport::new(selected_port, 115200)
            .timeout(std::time::Duration::from_millis(1000))
            .open().unwrap();
            while !clone_.load(std::sync::atomic::Ordering::Relaxed) 
            {
                let send_packet = match node.subscriber.recv()
                {
                    Ok(ok)=>{
                        ok
                    }
                    Err(_e)=>{
                        let p = Packet{x:10, y:10, ro:10, m1:10, m2:10};

                        p
                    }
                };

                let write_buf = if is_
                {
                    format!("s{},{},{},{}e",
                            send_packet.x/10 as i32+10,
                            send_packet.y/10 as i32+10,
                            send_packet.ro/10 as i32+10,
                            send_packet.m1/10 as i32+10)
                }
                else
                {
                    format!("s{},{},{},{},{}e",
                            send_packet.x/10 as i32+10,
                            send_packet.y/10 as i32+10,
                            send_packet.ro/10 as i32+10,
                            send_packet.m1/10 as i32+10,
                            send_packet.m2/10 as i32+10)
                };

                match port_.write(write_buf.as_bytes()) {
                    Ok(_)=>{
                        println!("Write:{}", write_buf);
                        let _ = port_.clear(serialport::ClearBuffer::Input);
                    }
                    Err(e)=>{
                        println!("{:?}", e);
                        let _ = port_.clear(serialport::ClearBuffer::Output);
                    }
                }
            }
        });
    }
    pub fn spawn_smooth_serial(&mut self, smooth_value:i32)
    {
        let selected_port = self.selected.clone();
        let node = ThreadConnector::<Packet>::new();
        self.conn[self.driver_num].publisher = node.publisher.clone();
        let clone_ = self.thread_manager[self.driver_num].get_clone();
        self.id.push(self.driver_num);
        self.id_box = ComboBox::new(self.id.clone());

        self.driver_num += 1;
        self.thread_manager.push(ThreadManager::new());
        self.conn.push(ThreadConnector::<Packet>::new());
        let is_ = self.is_small_packet.clone();

        std::thread::spawn(move ||{
            let mut port_ = serialport::new(selected_port, 115200)
            .timeout(std::time::Duration::from_millis(1000))
            .open().unwrap();

            let mut send = Packet{x:100, y:100, ro:100, m1:100, m2:100};
            let mut history = Packet{x:100, y:100, ro:100, m1:100, m2:100};
            while !clone_.load(std::sync::atomic::Ordering::Relaxed) 
            {
                let target = match node.subscriber.recv()
                {
                    Ok(ok)=>{
                        ok
                    }
                    Err(_e)=>{
                        let p = Packet{x:0, y:0, ro:0, m1:0, m2:0};

                        p
                    }
                };

                let vec = Packet{
                    x: target.x - history.x,
                    y: target.y - history.y,
                    ro: target.ro - history.ro,
                    m1: target.m1 - history.m1,
                    m2: target.m2 - history.m2,
                };

                if vec.x > 0
                {
                    send.x += smooth_value;
                }
                else if vec.x < 0
                {
                    send.x -= smooth_value;
                }

                if vec.y > 0
                {
                    send.y += smooth_value
                }
                else if vec.y < 0
                {
                    send.y -= smooth_value;
                }

                if vec.ro > 0
                {
                    send.ro += smooth_value;
                }
                else if vec.ro < 0
                {
                    send.ro -= smooth_value;
                }

                if vec.m1 > 0
                {
                    send.m1 += smooth_value;
                }
                else if vec.m1 < 0
                {
                    send.m1 -= smooth_value;
                }

                if vec.m2 > 0
                {
                    send.m2 += smooth_value;
                }
                else if vec.m2 < 0
                {
                    send.m2 -= smooth_value;
                }

                let write_buf = if is_
                {
                    format!("s{},{},{},{}e",
                            (send.x/10) as i32+10,
                            (send.y/10) as i32+10,
                            (send.ro/10) as i32+10,
                            (send.m1/10) as i32+10)
                }
                else
                {
                    format!("s{},{},{},{},{}e",
                            (send.x/10) as i32+10,
                            (send.y/10) as i32+10,
                            (send.ro/10) as i32+10,
                            (send.m1/10) as i32+10,
                            (send.m2/10) as i32+10)
                };

                match port_.write(write_buf.as_bytes()) {
                    Ok(_)=>{
                        println!("Write:{}", write_buf);
                        let _ = port_.clear(serialport::ClearBuffer::Input);
                    }
                    Err(e)=>{
                        println!("{:?}", e);
                        let _ = port_.clear(serialport::ClearBuffer::Output);
                    }
                }

                history.x = send.x as i32;
                history.y = send.y as i32;
                history.ro = send.ro as i32;
                history.m1 = send.m1 as i32;
                history.m2 = send.m2 as i32;
            }

            drop(port_);
        });

        println!("Stop SmoothSerial path:{}", self.selected.clone());
    }
}