pub mod logging;

// با استفاده از pub(crate) سطح دسترسی ماکروها را دقیقاً مطابق معماری اصلی حفظ می‌کنیم.
// این خط به تنهایی تمام ارورهای ambiguous (E0659) و unresolved (E0432) را برطرف می‌کند.
pub(crate) use logging::{critical, debug, error, info, message, warning};

pub mod platform;
pub mod utils;