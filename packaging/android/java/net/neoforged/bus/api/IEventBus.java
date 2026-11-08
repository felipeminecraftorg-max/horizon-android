// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.bus.api;

import java.util.function.Consumer;

/** Where events are posted and heard (lane 3's is {@code dev.horizon.android.lane3.Neo.Bus}). */
public interface IEventBus {
    void register(Object target);

    <T extends Event> void addListener(Consumer<T> consumer);

    <T extends Event> void addListener(Class<T> type, Consumer<T> consumer);

    <T extends Event> void addListener(EventPriority priority, Consumer<T> consumer);

    <T extends Event> void addListener(EventPriority priority, Class<T> type, Consumer<T> consumer);

    <T extends Event> void addListener(EventPriority priority, boolean receiveCanceled, Consumer<T> consumer);

    <T extends Event> void addListener(EventPriority priority, boolean receiveCanceled, Class<T> type, Consumer<T> consumer);

    <T extends Event> void addListener(boolean receiveCanceled, Consumer<T> consumer);

    <T extends Event> void addListener(boolean receiveCanceled, Class<T> type, Consumer<T> consumer);

    void unregister(Object target);

    <T extends Event> T post(T event);

    void start();
}
