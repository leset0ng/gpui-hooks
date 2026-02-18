use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, Window, WindowBounds,
    WindowOptions,
};
use gpui_hooks::{hook_element, HookedRender};
// 按需导入需要的 hook traits
use gpui_hooks::hooks::{UseEffectHook, UseMemoHook, UseStateHook};

#[hook_element]
struct CounterApp {}

impl HookedRender for CounterApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // useState - 管理计数器状态
        let (count, set_count) = self.use_state(|| 0i32);

        // useMemo - 计算双倍值
        let count_val = count();
        let doubled = self.use_memo([count_val], || count_val * 2);

        // useEffect - 副作用，当count变化时执行
        self.use_effect([count_val], || {
            println!("Effect: count changed to {}", count_val);
            // 返回可选的清理函数
            Some(|| {
                println!("Effect cleanup: previous effect is being cleaned up");
            })
        });

        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x2d2d2d))
            .size(px(500.0))
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x555555))
            .text_xl()
            .text_color(rgb(0xffffff))
            // 计数器显示
            .child(format!("Count: {}", count()))
            // 双倍值显示（useMemo）
            .child(format!("Doubled (useMemo): {}", doubled()))
            // 操作说明
            .child("Check console for effect logs")
            .child(div().child("click me").id("counter").on_click(cx.listener(
                move |_this, _, _window, cx| {
                    set_count(count() + 1);
                    cx.notify();
                },
            )))
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| CounterApp {
                    ..Default::default()
                })
            },
        )
        .unwrap();
    });
}
