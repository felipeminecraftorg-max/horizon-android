// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.fabricmc.fabric.api.event;

import java.lang.reflect.Array;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.function.Function;

public final class EventFactory {
    private static final List<ArrayBacked<?>> ALL = new ArrayList<>();

    private EventFactory() {}

    public static <T> Event<T> createArrayBacked(Class<? super T> type, Function<T[], T> invokerFactory) {
        ArrayBacked<T> event = new ArrayBacked<>(type, invokerFactory);
        synchronized (ALL) {
            ALL.add(event);
        }
        return event;
    }

    public static <T> Event<T> createArrayBacked(
            Class<T> type, T emptyInvoker, Function<T[], T> invokerFactory) {
        return createArrayBacked(type, listeners -> listeners.length == 0 ? emptyInvoker : invokerFactory.apply(listeners));
    }

    public static boolean isProfilingEnabled() {
        return false;
    }

    /** Lane 3 starts every game with no listener left from a check or an earlier world. */
    public static void clearAll() {
        synchronized (ALL) {
            for (ArrayBacked<?> e : ALL) {
                e.clear();
            }
        }
    }

    static final class ArrayBacked<T> extends Event<T> {
        private final Class<? super T> type;
        private final Function<T[], T> factory;
        private T[] handlers;

        @SuppressWarnings("unchecked")
        ArrayBacked(Class<? super T> type, Function<T[], T> factory) {
            this.type = type;
            this.factory = factory;
            this.handlers = (T[]) Array.newInstance(type, 0);
            update();
        }

        private void update() {
            invoker = factory.apply(handlers);
        }

        @Override
        public synchronized void register(T listener) {
            if (listener == null) {
                throw new NullPointerException("Tried to register a null listener");
            }
            handlers = Arrays.copyOf(handlers, handlers.length + 1);
            handlers[handlers.length - 1] = listener;
            update();
            dev.horizon.android.lane3.Lane3.registered(type.getName());
        }

        @SuppressWarnings("unchecked")
        synchronized void clear() {
            handlers = (T[]) Array.newInstance(type, 0);
            update();
        }
    }
}
