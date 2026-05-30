<div align="center">

# 🔄 MarkItDown-RST

**Многоформатный конвертер документов в Markdown с OCR и мультиязычным UI**

[![Build Apps](https://github.com/AlexZander85/markitdown-rst/actions/workflows/build.yml/badge.svg)](https://github.com/AlexZander85/markitdown-rst/actions/workflows/build.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

[🇷🇺 Русский](#-русский) · [🇬🇧 English](#-english) · [🇨🇳 中文](#-中文)

</div>

---

## 🇷🇺 Русский

### Что такое MarkItDown-RST?

**MarkItDown-RST** — это высокопроизводительное десктоп-приложение на Rust, которое конвертирует документы 13+ форматов в чистый Markdown. Работает в десятки раз быстрее Python-аналогов благодаря многопоточной архитектуре на Tokio + Rayon, а встроенный Tesseract OCR распознаёт текст на изображениях прямо из коробки — без внешних зависимостей.

### ✨ Ключевые возможности

#### 🚀 Скорость и многопоточность
- **Tokio + Semaphore** — параллельная конвертация нескольких файлов одновременно с настраиваемым числом потоков
- **Rayon** — параллельная обработка данных внутри конвертеров
- **Автоопределение CPU** — по умолчанию использует все доступные ядра процессора
- Пакетная обработка сотен файлов за секунды с прогресс-баром и статистикой

#### 📄 Поддерживаемые форматы (13+)

| Формат | Вход | Выход | Движок | Особенности |
|--------|------|-------|--------|-------------|
| **PDF** | `.pdf` | Markdown, JSON | pdf-extract + lopdf | Постраничное извлечение, определение заголовков |
| **DOCX** | `.docx` | Markdown, JSON | ZIP + quick-xml | Жирный, курсив, заголовки, таблицы, разрывы |
| **XLSX** | `.xlsx` | Markdown-таблицы, CSV, JSON | umya-spreadsheet | Мультилистовые таблицы, пустые листы |
| **PPTX** | `.pptx` | Markdown по слайдам, JSON | ZIP + quick-xml | Извлечение текста из слайдов и фигур |
| **HTML** | `.html`, `.htm` | Markdown, JSON | html2text | Извлечение заголовка `<title>` |
| **XML** | `.xml` | Markdown, JSON | quick-xml | Глубинное форматирование с заголовками по уровням |
| **TXT** | `.txt`, `.md` | Markdown, JSON | Эвристика | Автоопределение заголовков по регистру и длине |
| **CSV/TSV** | `.csv`, `.tsv` | Markdown-таблицы, JSON | csv crate | Автоопределение разделителя |
| **RTF** | `.rtf` | Markdown, JSON | Regex-парсер | Упрощённое извлечение текста (Beta) |
| **ODT** | `.odt` | Markdown, JSON | ZIP + XML | Извлечение абзацев, заголовков, span |
| **Изображения** | `.jpg`, `.png`, `.tiff`, `.bmp`, `.gif`, `.webp` | Markdown (OCR), JSON | Tesseract | 3 языка: английский, русский, китайский |
| **ZIP** | `.zip` | Индекс, статистика, Markdown, JSON | zip crate | Анализ содержимого, извлечение текстовых файлов до 50KB |
| **JSON** | `.json` | Pretty-print в блоке кода, таблица | serde_json | Автотаблица для массивов объектов |

#### 🔍 Встроенный OCR (Tesseract)
- **Tesseract OCR** интегрирован в приложение — не нужно устанавливать отдельно
- **tessdata_fast** для 3 языков зашита прямо в бинарник через `include_bytes!`:
  - 🇬🇧 Английский (~4 MB)
  - 🇷🇺 Русский (~3.7 MB)
  - 🇨🇳 Китайский упрощённый (~2.4 MB)
- Выбор языков распознавания чекбоксами в GUI или флагом `--ocr-langs` в CLI
- Автоматическое извлечение tessdata в директорию приложения при первом запуске

#### 👁️ Просмотр Markdown (как mdhero)
- Конвертация MD → HTML через **comrak** с открытием в браузере
- **highlight.js** — подсветка синтаксиса для 25+ языков программирования
- **KaTeX** — рендеринг математических формул (`$E=mc^2$`, `$$\int_0^\infty$$`)
- **Mermaid** — визуализация диаграмм и блок-схем
- Apple-inspired типографика с засечками и чистым дизайном
- **Светлая / тёмная тема** с мгновенным переключением
- Плавающая панель управления: размер шрифта, высота строк, ширина контента
- Адаптивная вёрстка, стили для печати, кастомные скроллбары

#### 🌍 Мультиязычный интерфейс (3 языка)
- 🇬🇧 **English** — полный перевод
- 🇷🇺 **Русский** — полный перевод
- 🇨🇳 **中文** — полный перевод
- 35+ строк интерфейса переведены на каждый язык
- Переключение языка в один клик в верхней панели

#### 🖥️ Два режима работы

**GUI (Графический интерфейс):**
- Нативный интерфейс на **egui/eframe** — быстрый, легковесный, отзывчивый
- Drag & Drop — перетаскивание файлов и папок прямо в окно
- Очередь файлов с иконками статуса (⏳ → 🔄 → ✅ / ❌)
- Настройка потоков, выходной директории, объединённого вывода
- Чекбоксы языков OCR
- Предпросмотр Markdown с моноширинным шрифтом
- Кнопки «Открыть в браузере», «Сохранить», «Копировать»
- Тёмная / светлая тема

**CLI (Командная строка):**
- `markitdown-cli convert <файл>` — конвертация одного файла
- `markitdown-cli batch <папка>` — пакетная конвертация с прогресс-баром
- `markitdown-cli info <файл>` — информация о документе
- `markitdown-cli formats` — список поддерживаемых форматов
- `markitdown-cli ocr-check` — проверка доступности Tesseract и tessdata
- Цветной вывод, флаг `--optimize-llm` для оптимизации под LLM

#### ⚙️ Компактный бинарник
- **LTO + strip + panic=abort** — минимальный размер бинарника
- Один исполняемый файл — никаких внешних зависимостей
- Feature flags для сборки без лишнего:
  - `default` = GUI + OCR + Preview (полная версия)
  - `cli-only` = только CLI, без GUI
  - `light` = только конвертация, без OCR и просмотра
  - `full` = все функции

### 📦 Установка

Скачайте бинарник для вашей ОС во вкладке [Actions → Artifacts](https://github.com/AlexZander85/markitdown-rst/actions) или соберите самостоятельно:

```bash
# Полная версия (GUI + OCR + Preview)
git clone https://github.com/AlexZander85/markitdown-rst.git
cd markitdown-rst
cargo build --release

# Только CLI (без GUI)
cargo build --release --no-default-features --features cli-only

# Лёгкая версия (GUI без OCR и Preview)
cargo build --release --no-default-features --features gui
```

### 🏗️ Архитектура

```
markitdown-rst/
├── src/
│   ├── main.rs           # Точка входа GUI
│   ├── bin/cli.rs        # CLI-интерфейс (clap)
│   ├── lib.rs            # Корень библиотеки
│   ├── converters/       # 11 конвертеров форматов
│   │   ├── mod.rs        # Трейт DocumentConverter, фабрика
│   │   ├── pdf.rs        # PDF → MD (pdf-extract + lopdf)
│   │   ├── docx.rs       # DOCX → MD (ZIP + XML парсинг)
│   │   ├── xlsx.rs       # XLSX → MD-таблицы (umya-spreadsheet)
│   │   ├── pptx.rs       # PPTX → MD по слайдам (ZIP + XML)
│   │   ├── html.rs       # HTML → MD (html2text) + XML → MD
│   │   ├── csv.rs        # CSV/TSV → MD-таблицы
│   │   ├── txt.rs        # TXT/RTF/ODT → MD
│   │   ├── json_conv.rs  # JSON → pretty-print + таблица
│   │   ├── zip_conv.rs   # ZIP → индекс + статистика
│   │   └── image_ocr.rs  # Изображения → MD через OCR
│   ├── gui/
│   │   ├── mod.rs        # eframe NativeOptions
│   │   └── app.rs        # Полное GUI-приложение (egui)
│   ├── ocr/
│   │   └── mod.rs        # Tesseract wrapper + встроенные tessdata
│   ├── preview/
│   │   └── mod.rs        # MD→HTML с hljs/KaTeX/Mermaid + CSS/JS
│   ├── i18n/
│   │   └── mod.rs        # 35+ переводов (EN/RU/ZH)
│   ├── batch/
│   │   └── mod.rs        # Tokio Semaphore BatchProcessor
│   └── utils/
│       └── mod.rs        # InputFormat, OutputFormat, detect, helpers
├── tessdata/             # Встроенные модели Tesseract
│   ├── eng.traineddata
│   ├── rus.traineddata
│   └── chi_sim.traineddata
├── .github/workflows/
│   └── build.yml         # CI/CD для Linux/macOS/Windows
└── Cargo.toml
```

### 🔧 Требования для сборки

**Linux:**
```bash
sudo apt install libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libx11-dev libx11-xcb-dev libxrandr-dev libxi-dev libxcursor-dev \
  libxdamage-dev libxfixes-dev libxinerama-dev libwayland-dev \
  libgbm-dev libegl-dev libclang-dev libspeechd-dev \
  libtesseract-dev libleptonica-dev
```

**macOS:**
```bash
brew install tesseract leptonica
```

**Windows:** никаких дополнительных зависимостей, только Rust toolchain.

### 📄 Лицензия

MIT License — используйте свободно.

---

## 🇬🇧 English

### What is MarkItDown-RST?

**MarkItDown-RST** is a high-performance Rust desktop application that converts 13+ document formats into clean Markdown. It runs dozens of times faster than Python alternatives thanks to a multithreaded Tokio + Rayon architecture, and the built-in Tesseract OCR recognizes text in images out of the box — no external dependencies required.

### ✨ Key Features

#### 🚀 Speed & Multithreading
- **Tokio + Semaphore** — parallel conversion of multiple files with configurable thread count
- **Rayon** — parallel data processing inside converters
- **Auto CPU detection** — uses all available CPU cores by default
- Batch processing of hundreds of files in seconds with progress bars and statistics

#### 📄 Supported Formats (13+)

| Format | Input | Output | Engine | Features |
|--------|-------|--------|--------|----------|
| **PDF** | `.pdf` | Markdown, JSON | pdf-extract + lopdf | Page-by-page extraction, heading detection |
| **DOCX** | `.docx` | Markdown, JSON | ZIP + quick-xml | Bold, italic, headings, tables, breaks |
| **XLSX** | `.xlsx` | Markdown tables, CSV, JSON | umya-spreadsheet | Multi-sheet tables, empty sheets |
| **PPTX** | `.pptx` | Per-slide Markdown, JSON | ZIP + quick-xml | Text from slides and shapes |
| **HTML** | `.html`, `.htm` | Markdown, JSON | html2text | `<title>` extraction |
| **XML** | `.xml` | Markdown, JSON | quick-xml | Depth-aware formatting with level headings |
| **TXT** | `.txt`, `.md` | Markdown, JSON | Heuristics | Auto heading detection by case & length |
| **CSV/TSV** | `.csv`, `.tsv` | Markdown tables, JSON | csv crate | Auto delimiter detection |
| **RTF** | `.rtf` | Markdown, JSON | Regex parser | Simplified text extraction (Beta) |
| **ODT** | `.odt` | Markdown, JSON | ZIP + XML | Paragraphs, headings, spans |
| **Images** | `.jpg`, `.png`, `.tiff`, `.bmp`, `.gif`, `.webp` | Markdown (OCR), JSON | Tesseract | 3 languages: English, Russian, Chinese |
| **ZIP** | `.zip` | Index, statistics, Markdown, JSON | zip crate | Content analysis, text extraction up to 50KB |
| **JSON** | `.json` | Pretty-print in code block, table | serde_json | Auto-table for object arrays |

#### 🔍 Built-in OCR (Tesseract)
- **Tesseract OCR** is integrated into the app — no separate installation needed
- **tessdata_fast** for 3 languages embedded directly into the binary via `include_bytes!`:
  - 🇬🇧 English (~4 MB)
  - 🇷🇺 Russian (~3.7 MB)
  - 🇨🇳 Simplified Chinese (~2.4 MB)
- Select OCR languages via checkboxes in GUI or `--ocr-langs` flag in CLI
- Automatic tessdata extraction to app data directory on first run

#### 👁️ Markdown Preview (mdhero-style)
- MD → HTML conversion via **comrak** with browser preview
- **highlight.js** — syntax highlighting for 25+ programming languages
- **KaTeX** — math formula rendering (`$E=mc^2$`, `$$\int_0^\infty$$`)
- **Mermaid** — diagram and flowchart visualization
- Apple-inspired typography with serifs and clean design
- **Light / Dark theme** with instant toggle
- Floating toolbar: font size, line height, content width
- Responsive layout, print styles, custom scrollbars

#### 🌍 Multilingual Interface (3 languages)
- 🇬🇧 **English** — full translation
- 🇷🇺 **Русский** — full translation
- 🇨🇳 **中文** — full translation
- 35+ UI strings translated into each language
- One-click language switching in the top panel

#### 🖥️ Two Operating Modes

**GUI (Graphical Interface):**
- Native UI built with **egui/eframe** — fast, lightweight, responsive
- Drag & Drop — drag files and folders directly into the window
- File queue with status icons (⏳ → 🔄 → ✅ / ❌)
- Thread count, output directory, combined output settings
- OCR language checkboxes
- Markdown preview with monospace font
- "Open in Browser", "Save", "Copy" buttons
- Dark / Light theme

**CLI (Command Line):**
- `markitdown-cli convert <file>` — single file conversion
- `markitdown-cli batch <folder>` — batch conversion with progress bar
- `markitdown-cli info <file>` — document information
- `markitdown-cli formats` — list supported formats
- `markitdown-cli ocr-check` — check Tesseract and tessdata availability
- Colored output, `--optimize-llm` flag for LLM optimization

#### ⚙️ Compact Binary
- **LTO + strip + panic=abort** — minimal binary size
- Single executable — no external dependencies
- Feature flags for lean builds:
  - `default` = GUI + OCR + Preview (full version)
  - `cli-only` = CLI only, no GUI
  - `light` = conversion only, no OCR or preview
  - `full` = all features

### 📦 Installation

Download the binary for your OS from [Actions → Artifacts](https://github.com/AlexZander85/markitdown-rst/actions) or build from source:

```bash
# Full version (GUI + OCR + Preview)
git clone https://github.com/AlexZander85/markitdown-rst.git
cd markitdown-rst
cargo build --release

# CLI only (no GUI)
cargo build --release --no-default-features --features cli-only

# Light version (GUI without OCR and Preview)
cargo build --release --no-default-features --features gui
```

### 🏗️ Architecture

```
markitdown-rst/
├── src/
│   ├── main.rs           # GUI entry point
│   ├── bin/cli.rs        # CLI interface (clap)
│   ├── lib.rs            # Library root
│   ├── converters/       # 11 format converters
│   │   ├── mod.rs        # DocumentConverter trait, factory
│   │   ├── pdf.rs        # PDF → MD (pdf-extract + lopdf)
│   │   ├── docx.rs       # DOCX → MD (ZIP + XML parsing)
│   │   ├── xlsx.rs       # XLSX → MD tables (umya-spreadsheet)
│   │   ├── pptx.rs       # PPTX → MD per slide (ZIP + XML)
│   │   ├── html.rs       # HTML → MD (html2text) + XML → MD
│   │   ├── csv.rs        # CSV/TSV → MD tables
│   │   ├── txt.rs        # TXT/RTF/ODT → MD
│   │   ├── json_conv.rs  # JSON → pretty-print + table
│   │   ├── zip_conv.rs   # ZIP → index + statistics
│   │   └── image_ocr.rs  # Images → MD via OCR
│   ├── gui/
│   │   ├── mod.rs        # eframe NativeOptions
│   │   └── app.rs        # Full GUI application (egui)
│   ├── ocr/
│   │   └── mod.rs        # Tesseract wrapper + embedded tessdata
│   ├── preview/
│   │   └── mod.rs        # MD→HTML with hljs/KaTeX/Mermaid + CSS/JS
│   ├── i18n/
│   │   └── mod.rs        # 35+ translations (EN/RU/ZH)
│   ├── batch/
│   │   └── mod.rs        # Tokio Semaphore BatchProcessor
│   └── utils/
│       └── mod.rs        # InputFormat, OutputFormat, detect, helpers
├── tessdata/             # Embedded Tesseract models
│   ├── eng.traineddata
│   ├── rus.traineddata
│   └── chi_sim.traineddata
├── .github/workflows/
│   └── build.yml         # CI/CD for Linux/macOS/Windows
└── Cargo.toml
```

### 🔧 Build Requirements

**Linux:**
```bash
sudo apt install libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libx11-dev libx11-xcb-dev libxrandr-dev libxi-dev libxcursor-dev \
  libxdamage-dev libxfixes-dev libxinerama-dev libwayland-dev \
  libgbm-dev libegl-dev libclang-dev libspeechd-dev \
  libtesseract-dev libleptonica-dev
```

**macOS:**
```bash
brew install tesseract leptonica
```

**Windows:** no additional dependencies, just the Rust toolchain.

### 📄 License

MIT License — use freely.

---

## 🇨🇳 中文

### MarkItDown-RST 是什么？

**MarkItDown-RST** 是一款高性能 Rust 桌面应用程序，可将 13+ 种文档格式转换为整洁的 Markdown。得益于 Tokio + Rayon 多线程架构，其运行速度比 Python 替代方案快数十倍。内置 Tesseract OCR 可开箱即用地识别图像中的文字——无需外部依赖。

### ✨ 核心功能

#### 🚀 速度与多线程
- **Tokio + Semaphore** — 可配置线程数的并行文件转换
- **Rayon** — 转换器内部并行数据处理
- **自动 CPU 检测** — 默认使用所有可用的 CPU 核心
- 批量处理数百个文件仅需数秒，带进度条和统计信息

#### 📄 支持格式（13+）

| 格式 | 输入 | 输出 | 引擎 | 特性 |
|------|------|------|------|------|
| **PDF** | `.pdf` | Markdown、JSON | pdf-extract + lopdf | 逐页提取、标题检测 |
| **DOCX** | `.docx` | Markdown、JSON | ZIP + quick-xml | 粗体、斜体、标题、表格、换行 |
| **XLSX** | `.xlsx` | Markdown 表格、CSV、JSON | umya-spreadsheet | 多工作表、空工作表处理 |
| **PPTX** | `.pptx` | 按幻灯片 Markdown、JSON | ZIP + quick-xml | 从幻灯片和形状提取文本 |
| **HTML** | `.html`、`.htm` | Markdown、JSON | html2text | 提取 `<title>` 标题 |
| **XML** | `.xml` | Markdown、JSON | quick-xml | 深度感知格式化，按级别标题 |
| **TXT** | `.txt`、`.md` | Markdown、JSON | 启发式算法 | 根据大小写和长度自动检测标题 |
| **CSV/TSV** | `.csv`、`.tsv` | Markdown 表格、JSON | csv crate | 自动分隔符检测 |
| **RTF** | `.rtf` | Markdown、JSON | 正则解析器 | 简化文本提取（Beta） |
| **ODT** | `.odt` | Markdown、JSON | ZIP + XML | 段落、标题、span 提取 |
| **图像** | `.jpg`、`.png`、`.tiff`、`.bmp`、`.gif`、`.webp` | Markdown（OCR）、JSON | Tesseract | 3种语言：英语、俄语、中文 |
| **ZIP** | `.zip` | 索引、统计、Markdown、JSON | zip crate | 内容分析、文本提取（最大50KB） |
| **JSON** | `.json` | 代码块中的 Pretty-print、表格 | serde_json | 对象数组自动生成表格 |

#### 🔍 内置 OCR（Tesseract）
- **Tesseract OCR** 已集成到应用程序中——无需单独安装
- 3种语言的 **tessdata_fast** 通过 `include_bytes!` 直接嵌入二进制文件：
  - 🇬🇧 英语（~4 MB）
  - 🇷🇺 俄语（~3.7 MB）
  - 🇨🇳 简体中文（~2.4 MB）
- 在 GUI 中通过复选框选择 OCR 语言，或在 CLI 中使用 `--ocr-langs` 标志
- 首次运行时自动提取 tessdata 到应用程序数据目录

#### 👁️ Markdown 预览（mdhero 风格）
- 通过 **comrak** 将 MD → HTML，在浏览器中预览
- **highlight.js** — 25+ 种编程语言的语法高亮
- **KaTeX** — 数学公式渲染（`$E=mc^2$`、`$$\int_0^\infty$$`）
- **Mermaid** — 图表和流程图可视化
- Apple 风格排版，衬线字体，简洁设计
- **浅色 / 深色主题** 即时切换
- 浮动工具栏：字体大小、行高、内容宽度
- 响应式布局、打印样式、自定义滚动条

#### 🌍 多语言界面（3种语言）
- 🇬🇧 **English** — 完整翻译
- 🇷🇺 **Русский** — 完整翻译
- 🇨🇳 **中文** — 完整翻译
- 35+ 条界面字符串已翻译为每种语言
- 顶部面板一键切换语言

#### 🖥️ 两种运行模式

**GUI（图形界面）：**
- 基于 **egui/eframe** 的原生界面——快速、轻量、响应迅速
- 拖放支持——直接将文件和文件夹拖入窗口
- 文件队列带状态图标（⏳ → 🔄 → ✅ / ❌）
- 线程数、输出目录、合并输出设置
- OCR 语言复选框
- 等宽字体的 Markdown 预览
- "在浏览器中打开"、"保存"、"复制" 按钮
- 深色 / 浅色主题

**CLI（命令行）：**
- `markitdown-cli convert <文件>` — 单文件转换
- `markitdown-cli batch <文件夹>` — 带进度条的批量转换
- `markitdown-cli info <文件>` — 文档信息
- `markitdown-cli formats` — 列出支持的格式
- `markitdown-cli ocr-check` — 检查 Tesseract 和 tessdata 可用性
- 彩色输出，`--optimize-llm` 标志用于 LLM 优化

#### ⚙️ 紧凑二进制文件
- **LTO + strip + panic=abort** — 最小二进制文件大小
- 单个可执行文件——无外部依赖
- 特性标志用于精简构建：
  - `default` = GUI + OCR + 预览（完整版）
  - `cli-only` = 仅 CLI，无 GUI
  - `light` = 仅转换，无 OCR 和预览
  - `full` = 所有功能

### 📦 安装

从 [Actions → Artifacts](https://github.com/AlexZander85/markitdown-rst/actions) 下载适合您操作系统的二进制文件，或从源代码构建：

```bash
# 完整版（GUI + OCR + 预览）
git clone https://github.com/AlexZander85/markitdown-rst.git
cd markitdown-rst
cargo build --release

# 仅 CLI（无 GUI）
cargo build --release --no-default-features --features cli-only

# 轻量版（GUI 无 OCR 和预览）
cargo build --release --no-default-features --features gui
```

### 🏗️ 架构

```
markitdown-rst/
├── src/
│   ├── main.rs           # GUI 入口
│   ├── bin/cli.rs        # CLI 界面 (clap)
│   ├── lib.rs            # 库根目录
│   ├── converters/       # 11 个格式转换器
│   │   ├── mod.rs        # DocumentConverter trait, 工厂
│   │   ├── pdf.rs        # PDF → MD (pdf-extract + lopdf)
│   │   ├── docx.rs       # DOCX → MD (ZIP + XML 解析)
│   │   ├── xlsx.rs       # XLSX → MD 表格 (umya-spreadsheet)
│   │   ├── pptx.rs       # PPTX → MD 按幻灯片 (ZIP + XML)
│   │   ├── html.rs       # HTML → MD (html2text) + XML → MD
│   │   ├── csv.rs        # CSV/TSV → MD 表格
│   │   ├── txt.rs        # TXT/RTF/ODT → MD
│   │   ├── json_conv.rs  # JSON → pretty-print + 表格
│   │   ├── zip_conv.rs   # ZIP → 索引 + 统计
│   │   └── image_ocr.rs  # 图像 → MD 通过 OCR
│   ├── gui/
│   │   ├── mod.rs        # eframe NativeOptions
│   │   └── app.rs        # 完整 GUI 应用 (egui)
│   ├── ocr/
│   │   └── mod.rs        # Tesseract 包装器 + 嵌入式 tessdata
│   ├── preview/
│   │   └── mod.rs        # MD→HTML 含 hljs/KaTeX/Mermaid + CSS/JS
│   ├── i18n/
│   │   └── mod.rs        # 35+ 翻译 (EN/RU/ZH)
│   ├── batch/
│   │   └── mod.rs        # Tokio Semaphore 批处理器
│   └── utils/
│       └── mod.rs        # InputFormat, OutputFormat, 检测, 辅助函数
├── tessdata/             # 嵌入式 Tesseract 模型
│   ├── eng.traineddata
│   ├── rus.traineddata
│   └── chi_sim.traineddata
├── .github/workflows/
│   └── build.yml         # Linux/macOS/Windows CI/CD
└── Cargo.toml
```

### 🔧 构建要求

**Linux:**
```bash
sudo apt install libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libx11-dev libx11-xcb-dev libxrandr-dev libxi-dev libxcursor-dev \
  libxdamage-dev libxfixes-dev libxinerama-dev libwayland-dev \
  libgbm-dev libegl-dev libclang-dev libspeechd-dev \
  libtesseract-dev libleptonica-dev
```

**macOS:**
```bash
brew install tesseract leptonica
```

**Windows:** 无需额外依赖，只需 Rust 工具链。

### 📄 许可证

MIT 许可证——自由使用。
