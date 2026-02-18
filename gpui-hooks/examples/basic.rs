use gpui::{
    App, Application, Bounds, Context, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};
use gpui_hooks::{HookedRender, hook_element};
// 按需导入需要的 hook traits
use gpui_hooks::hooks::{UseCallbackHook, UseEffectHook, UseMemoHook, UseRefHook, UseStateHook};

#[hook_element]
struct CounterApp {}

impl HookedRender for CounterApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // useState - 管理计数器状态
        let (count, set_count) = self.use_state(|| 0i32);

        // useMemo - 计算双倍值
        let count_val = count();
        let doubled = self.use_memo(|| count_val * 2, [count_val]);

        // useEffect - 副作用，当count变化时执行
        self.use_effect(
            || {
                println!("Effect: count changed to {}", count_val);
                // 返回可选的清理函数
                Some(|| {
                    println!("Effect cleanup: previous effect is being cleaned up");
                })
            },
            [count_val],
        );

        // useRef - 创建一个可变引用，用于存储上一次的值
        let prev_count_ref = self.use_ref(|| 0i32);
        let current_count = count_val;

        // 更新引用值
        if current_count != *prev_count_ref.borrow() {
            *prev_count_ref.borrow_mut() = current_count;
        }

        // useCallback - 创建一个记忆化的回调函数
        let handle_increment = self.use_callback(
            || {
                let count_val = current_count;
                Box::new(move || {
                    println!("Callback executed with count: {}", count_val);
                    count_val + 1
                }) as Box<dyn Fn() -> i32>
            },
            [current_count],
        );

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
            // useRef显示
            .child(format!(
                "Previous count (useRef): {}",
                *prev_count_ref.borrow()
            ))
            // 操作说明
            .child("Check console for effect and callback logs")
            .child(div().child("click me").id("counter").on_click(cx.listener(
                move |_this, _, _window, cx| {
                    set_count(count() + 1);
                    cx.notify();
                },
            )))
            // useCallback测试按钮
            .child(
                div()
                    .child("test callback")
                    .id("callback-test")
                    .on_click(cx.listener(move |_this, _, _window, _cx| {
                        let result = handle_increment();
                        println!("Callback returned: {}", result);
                    })),
            )
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
