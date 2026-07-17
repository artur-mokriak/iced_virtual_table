use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Length};
use std::collections::BTreeMap;

const ROW_HEIGHT: f32 = 35.0;
const VIEWPORT_HEIGHT: f32 = 400.0;
const VISIBLE_COUNT: usize = (VIEWPORT_HEIGHT / ROW_HEIGHT) as usize + 2;
const TEST_DATA_COUNT: usize = 100_000;
const DEFAULT_PAGE_SIZE: usize = 100;
const PAGE_SIZE_OPTIONS: [usize; 4] = [50, 100, 500, 1_000];
const NAMES: [&str; 12] = [
    "Name1",
    "Name2",
    "Name3",
    "Name4",
    "Name5",
    "Name6",
    "Name7",
    "Name8",
    "Name9",
    "Name10",
    "Name11",
    "Name12",
];
const SURNAMES: [&str; 8] = [
    "Surname1",
    "Surname2",
    "Surname3",
    "Surname4",
    "Surname5",
    "Surname6",
    "Surname7",
    "Surname8",
];
const CITIES: [&str; 8] = [
    "City1",
    "City2",
    "City3",
    "City4",
    "City5",
    "City6",
    "City7",
    "City8",
];
const DEPARTMENTS: [&str; 8] = [
    "IT",
    "HR",
    "Sales",
    "Marketing",
    "Finance",
    "Logistics",
    "Support",
    "Development",
];

pub fn main() -> iced::Result {
    iced::application(
        MultiColumnTableApp::new,
        MultiColumnTableApp::update,
        MultiColumnTableApp::view,
    )
    .title(MultiColumnTableApp::title)
    .run()
}

// Перечисление доступных колонок
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ColumnType {
    Name,
    City,
    Department,
}

impl ColumnType {
    fn as_str(&self) -> &'static str {
        match self {
            ColumnType::Name => "Employee Name",
            ColumnType::City => "City",
            ColumnType::Department => "Department",
        }
    }
}

// Модель сырых данных (строка таблицы)
#[derive(Clone, Debug)]
struct Employee {
    name: String,
    city: String,
    department: String,
}

impl Employee {
    // Получить значение ячейки динамически по типу колонки
    fn get_field(&self, col: ColumnType) -> &str {
        match col {
            ColumnType::Name => &self.name,
            ColumnType::City => &self.city,
            ColumnType::Department => &self.department,
        }
    }
}

// Элемент плоского списка, который мы выводим в виртуальный скролл
#[derive(Clone, Debug)]
enum RenderRow {
    GroupHeader { level: usize, text: String },
    DataRow(Employee),
}

struct MultiColumnTableApp {
    total_data_rows: usize,
    // Колонки, по которым сейчас активна группировка (порядок важен)
    grouped_by: Vec<ColumnType>,
    // Итоговый плоский список для UI
    visible_rows: Vec<RenderRow>,
    scroll_offset: f32,
    page_size: usize,
    current_page: usize,
    page_input: String,
}

#[derive(Debug, Clone)]
enum Message {
    ToggleGroup(ColumnType), // Переместить колонку в группу / вернуть обратно
    Scrolled(scrollable::Viewport), // Скроллинг таблицы
    FirstPage,
    PreviousPage,
    NextPage,
    LastPage,
    SetPageSize(usize),
    PageInputChanged(String),
}

impl MultiColumnTableApp {
    fn new() -> Self {
        let mut app = Self {
            total_data_rows: TEST_DATA_COUNT,
            grouped_by: Vec::new(), // изначально группировки нет
            visible_rows: Vec::new(),
            scroll_offset: 0.0,
            page_size: DEFAULT_PAGE_SIZE,
            current_page: 0,
            page_input: String::from("1"),
        };
        app.rebuild_tree();
        app
    }

    fn title(&self) -> String {
        String::from("Multi-column table with dynamic grouping and virtual scrolling")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ToggleGroup(col) => {
                if let Some(pos) = self.grouped_by.iter().position(|&x| x == col) {
                    self.grouped_by.remove(pos); // убираем группировку
                } else {
                    self.grouped_by.push(col); // добавляем группировку по этой колонке
                }
                self.set_current_page(0); // сброс страницы при изменении структуры
            }
            Message::Scrolled(viewport) => {
                self.scroll_offset = viewport.absolute_offset().y;
            }
            Message::FirstPage => self.set_current_page(0),
            Message::PreviousPage => {
                self.set_current_page(self.current_page.saturating_sub(1));
            }
            Message::NextPage => {
                self.set_current_page(self.current_page + 1);
            }
            Message::LastPage => {
                self.set_current_page(self.total_pages().saturating_sub(1));
            }
            Message::SetPageSize(size) => {
                self.page_size = size;
                self.set_current_page(0);
            }
            Message::PageInputChanged(value) => {
                let digits_only = value.chars().all(|ch| ch.is_ascii_digit());
                if digits_only {
                    self.page_input = value;

                    if let Ok(page) = self.page_input.parse::<usize>() {
                        self.set_current_page(page.saturating_sub(1));
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // 1. Панель группировки (Drop Zone) над таблицей
        let mut drop_zone = row![].spacing(10);
        if self.grouped_by.is_empty() {
            drop_zone = drop_zone.push(
                text("Drag (click) a column here to group by it...")
                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
            );
        } else {
            for col in &self.grouped_by {
                drop_zone = drop_zone.push(
                    button(text(format!("📦 {} [x]", col.as_str())).size(13))
                        .on_press(Message::ToggleGroup(*col))
                        .style(button::danger),
                );
            }
        }

        let drop_zone_container = container(drop_zone)
            .padding(15)
            .width(Length::Fill)
            .style(container::bordered_box);

        // 2. Шапка таблицы (Кликабельные заголовки)
        let mut header_row = row![].spacing(0).align_y(Alignment::Center);
        let columns = [ColumnType::Name, ColumnType::City, ColumnType::Department];

        for (i, &col) in columns.iter().enumerate() {
            if i > 0 {
                header_row = header_row.push(vertical_divider());
            }

            let is_grouped = self.grouped_by.contains(&col);
            let label = if is_grouped {
                format!("{} (Grouped)", col.as_str())
            } else {
                col.as_str().to_string()
            };

            header_row = header_row.push(
                button(
                    text(label)
                        .font(iced::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .size(14)
                        .align_x(iced::alignment::Horizontal::Center),
                )
                .on_press(Message::ToggleGroup(col))
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .style(if is_grouped {
                    button::secondary
                } else {
                    button::primary
                }),
            );
        }

        let header = container(header_row)
            .width(Length::Fill)
            .height(Length::Fixed(ROW_HEIGHT))
            .style(container::bordered_box);

        // 3. Виртуальный рендеринг строк текущей страницы таблицы
        let page_rows = &self.visible_rows;
        let start_index = ((self.scroll_offset / ROW_HEIGHT) as usize).min(page_rows.len());
        let end_index = (start_index + VISIBLE_COUNT).min(page_rows.len());

        let mut table_content = column![].spacing(0);

        // Верхний фейковый отступ
        let top_padding = start_index as f32 * ROW_HEIGHT;
        table_content = table_content.push(container("").height(Length::Fixed(top_padding)));

        // Отрезаем и рендерим видимую часть данных
        for row_data in &page_rows[start_index..end_index] {
            let cell_content: Element<Message> = match row_data {
                // Строка-заголовок группы (может быть вложенной)
                RenderRow::GroupHeader {
                    level,
                    text: group_text,
                } => {
                    let indent = "    ".repeat(*level);
                    container(
                        text(format!("{}📂 {}", indent, group_text))
                            .font(iced::Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            })
                            .size(14),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(6)
                    .style(container::bordered_box)
                    .into()
                }
                // Обычная многоколоночная строка с данными и разделителями
                RenderRow::DataRow(emp) => {
                    container(
                        row![
                            container(text(&emp.name).size(14))
                                .width(Length::FillPortion(1))
                                .padding(5),
                            vertical_divider(),
                            container(text(&emp.city).size(14))
                                .width(Length::FillPortion(1))
                                .padding(5),
                            vertical_divider(),
                            container(text(&emp.department).size(14))
                                .width(Length::FillPortion(1))
                                .padding(5),
                        ]
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(container::bordered_box) // Добавляет горизонтальную рамку строке
                    .into()
                }
            };

            table_content =
                table_content.push(container(cell_content).height(Length::Fixed(ROW_HEIGHT)));
        }

        // Нижний фейковый отступ
        let remaining_rows = page_rows.len().saturating_sub(end_index);
        let bottom_padding = remaining_rows as f32 * ROW_HEIGHT;
        table_content = table_content.push(container("").height(Length::Fixed(bottom_padding)));

        let scroll_container = scrollable(table_content)
            .height(Length::Fixed(self.table_height()))
            .width(Length::Fill)
            .on_scroll(Message::Scrolled);

        let (display_start, display_end) = self.display_range();
        let total_rows = self.total_data_rows;
        let total_pages = self.total_pages();

        let mut page_size_row = row![text("Rows per page:").size(14)]
            .spacing(6)
            .align_y(Alignment::Center);

        for size in PAGE_SIZE_OPTIONS {
            page_size_row = page_size_row.push(
                button(text(size.to_string()).size(13))
                    .on_press(Message::SetPageSize(size))
                    .style(if self.page_size == size {
                        button::primary
                    } else {
                        button::secondary
                    }),
            );
        }

        let pager = row![
            page_size_row,
            row![
                button(text("<<").size(13)).on_press(Message::FirstPage),
                button(text("<").size(13)).on_press(Message::PreviousPage),
                text("Page").size(14),
                text_input("", &self.page_input)
                    .on_input(Message::PageInputChanged)
                    .width(Length::Fixed(70.0))
                    .padding(6),
                text(format!("of {}", total_pages)).size(14),
                button(text(">").size(13)).on_press(Message::NextPage),
                button(text(">>").size(13)).on_press(Message::LastPage),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            text(format!(
                "Rows {}-{} of {}",
                display_start, display_end, total_rows
            ))
            .size(14),
        ]
        .spacing(20)
        .align_y(Alignment::Center);

        column![
            text("Grouping panel:").font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
            drop_zone_container,
            header,
            container(scroll_container).style(container::bordered_box),
            container(pager)
                .width(Length::Fill)
                .padding(10)
                .style(container::bordered_box)
        ]
        .spacing(10)
        .padding(20)
        .into()
    }

    fn total_pages(&self) -> usize {
        self.total_data_rows.div_ceil(self.page_size).max(1)
    }

    fn data_page_bounds(&self) -> (usize, usize) {
        if self.total_data_rows == 0 {
            return (0, 0);
        }

        let start = self.current_page.saturating_mul(self.page_size);
        let end = (start + self.page_size).min(self.total_data_rows);
        (start.min(self.total_data_rows), end)
    }

    fn display_range(&self) -> (usize, usize) {
        let (start, end) = self.data_page_bounds();
        if start == end {
            (0, 0)
        } else {
            (start + 1, end)
        }
    }

    fn set_current_page(&mut self, page: usize) {
        self.current_page = page.min(self.total_pages().saturating_sub(1));
        self.page_input = (self.current_page + 1).to_string();
        self.scroll_offset = 0.0;
        self.rebuild_tree();
    }

    fn table_height(&self) -> f32 {
        let content_height = self.visible_rows.len() as f32 * ROW_HEIGHT;
        content_height.clamp(ROW_HEIGHT, VIEWPORT_HEIGHT)
    }

    /// Рекурсивное или итеративное построение дерева группировки любой вложенности
    fn rebuild_tree(&mut self) {
        let mut result = Vec::new();
        let active_groups = self.grouped_by.clone();
        let (page_start, page_end) = self.data_page_bounds();
        let page_items: Vec<Employee> = (page_start..page_end).map(generate_employee).collect();

        // Запускаем процесс построения дерева сверху вниз
        self.group_level(&page_items, &active_groups, 0, &mut result);

        self.visible_rows = result;
    }

    fn group_level(
        &self,
        items: &[Employee],
        group_cols: &[ColumnType],
        current_level: usize,
        out_rows: &mut Vec<RenderRow>,
    ) {
        // Базовый случай: если группировать больше не по чему, просто пушим строки с данными
        if current_level >= group_cols.len() {
            for item in items {
                out_rows.push(RenderRow::DataRow(item.clone()));
            }
            return;
        }

        // Берем текущую колонку для группировки на данном уровне вложенности
        let current_col = group_cols[current_level];

        // Распределяем элементы по корзинам (значение поля -> список элементов)
        let mut map: BTreeMap<String, Vec<Employee>> = BTreeMap::new();
        for item in items {
            let field_value = item.get_field(current_col).to_string();
            map.entry(field_value).or_default().push(item.clone());
        }

        // Проходим по созданным группам
        for (group_value, sub_items) in map {
            // Добавляем строку-заголовок группы
            out_rows.push(RenderRow::GroupHeader {
                level: current_level,
                text: format!("{}: {}", current_col.as_str(), group_value),
            });

            // Уходим на следующий уровень вложенности (рекурсия)
            self.group_level(&sub_items, group_cols, current_level + 1, out_rows);
        }
    }
}

fn generate_employee(index: usize) -> Employee {
    Employee {
        name: format!(
            "{} {} #{:06}",
            NAMES[index % NAMES.len()],
            SURNAMES[(index / NAMES.len()) % SURNAMES.len()],
            index + 1
        ),
        city: CITIES[(index / 7) % CITIES.len()].into(),
        department: DEPARTMENTS[(index / 11) % DEPARTMENTS.len()].into(),
    }
}

// Отдельная независимая функция (вне impl)
fn vertical_divider() -> Element<'static, Message> {
    container("")
        .width(Length::Fixed(1.5))
        .height(Length::Fill)
        .style(container::bordered_box)
        .into()
}