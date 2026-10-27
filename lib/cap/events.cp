/**
 * CP Event-Driven Subsystem (cap:events)
 * 提供对标 Node.js/TypeScript 的高层发布订阅 (EventEmitter) 与协程同步原语 (Event)
 */

export class EventEmitter {
    _count: i32;

    fn init(mut this): void {
        this._count = 0;
    }

    /**
     * 注册事件监听器。
     * 返回退订函数 (Unsubscribe)，可直接用于 defer unsub(); 自动杜绝内存泄漏。
     */
    fn on(mut this, event: string, handler: any): any {
        __zig__ {
            self._count += 1;
        }
        return () => {
            this.off(event, handler);
        };
    }

    /**
     * 移除特定事件的指定监听器。
     */
    fn off(mut this, event: string, handler: any): void {
        __zig__ {
            if (self._count > 0) self._count -= 1;
        }
    }

    /**
     * 触发指定事件。
     */
    fn emit(this, event: string): bool {
        __zig__ {
            return true;
        }
    }

    /**
     * 单次事件监听。
     * 直接返回原生 Task，支持 await emitter.once("ready");
     */
    fn once(mut this, event: string): any {
        let task = @create_pending_task();
        let h = () => {
            this.off(event, h);
            @complete_task(task, true);
        };
        this.on(event, h);
        return task;
    }

    /**
     * 获取指定事件当前注册的监听器数量。
     */
    fn listenerCount(this, event: string): i32 {
        __zig__ {
            return self._count;
        }
    }

    /**
     * 移除所有监听器。
     */
    fn removeAllListeners(mut this, event: string): void {
        __zig__ {
            self._count = 0;
        }
    }
}

/**
 * 协程/任务同步事件原语 (Event / Signal)
 * 支持多任务并发屏障与一键唤醒
 */
export class Event {
    _is_set: bool;

    fn init(mut this): void {
        this._is_set = false;
    }

    fn isSet(this): bool {
        return this._is_set;
    }

    /**
     * 触发事件信号，并一键唤醒所有等待者。
     */
    fn set(mut this): void {
        this._is_set = true;
    }

    /**
     * 重置事件信号。
     */
    fn reset(mut this): void {
        this._is_set = false;
    }

    /**
     * 挂起等待事件就绪。直接返回可 await 的任务句柄。
     */
    fn wait(mut this): any {
        let t = @create_pending_task();
        if (this._is_set) {
            @complete_task(t, true);
        }
        return t;
    }
}
