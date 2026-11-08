// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package dev.horizon.android.lane3;

import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.function.Consumer;
import java.util.function.Supplier;
import net.minecraft.resources.ResourceKey;
import net.neoforged.bus.api.Event;
import net.neoforged.bus.api.EventPriority;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.fml.ModContainer;
import net.neoforged.fml.common.EventBusSubscriber;

/**
 * NeoForge's loader as lane 3 runs it: a mod's {@code @Mod} class is constructed with its own event
 * bus, its {@code @EventBusSubscriber} classes hear the game's bus, and what it registers is made
 * the first time the game asks for it.
 */
public final class Neo {
    private Neo() {}

    /** An event bus: listeners by the type of event they hear. */
    public static final class Bus implements IEventBus {
        private final List<Map.Entry<Class<?>, Consumer<Object>>> listeners = new ArrayList<>();

        @SuppressWarnings("unchecked")
        private void listen(Class<?> type, Consumer<?> consumer) {
            listeners.add(new java.util.AbstractMap.SimpleImmutableEntry<>(type == null ? Object.class : type, (Consumer<Object>) consumer));
        }

        @Override
        public void register(Object target) {
            boolean statics = target instanceof Class<?>;
            Class<?> type = statics ? (Class<?>) target : target.getClass();
            for (Method m : type.getDeclaredMethods()) {
                if (m.getParameterCount() != 1 || Modifier.isStatic(m.getModifiers()) != statics) continue;
                boolean subscribed;
                try {
                    subscribed = m.isAnnotationPresent(SubscribeEvent.class);
                } catch (Throwable t) {
                    subscribed = false;
                }
                if (!subscribed) continue;
                Class<?> event = m.getParameterTypes()[0];
                m.setAccessible(true);
                listen(event, e -> {
                    try {
                        m.invoke(statics ? null : target, e);
                    } catch (java.lang.reflect.InvocationTargetException x) {
                        Lane3.failed(type.getName() + "." + m.getName(), x.getCause());
                    } catch (Throwable x) {
                        Lane3.failed(type.getName() + "." + m.getName(), x);
                    }
                });
            }
        }

        // A listener given without its event's type hears every event it accepts: one of another
        // type fails its cast, and is not for it.
        @Override
        public <T extends Event> void addListener(Consumer<T> consumer) {
            listen(null, consumer);
        }

        @Override
        public <T extends Event> void addListener(Class<T> type, Consumer<T> consumer) {
            listen(type, consumer);
        }

        @Override
        public <T extends Event> void addListener(EventPriority priority, Consumer<T> consumer) {
            listen(null, consumer);
        }

        @Override
        public <T extends Event> void addListener(EventPriority priority, Class<T> type, Consumer<T> consumer) {
            listen(type, consumer);
        }

        @Override
        public <T extends Event> void addListener(EventPriority priority, boolean receiveCanceled, Consumer<T> consumer) {
            listen(null, consumer);
        }

        @Override
        public <T extends Event> void addListener(EventPriority priority, boolean receiveCanceled, Class<T> type, Consumer<T> consumer) {
            listen(type, consumer);
        }

        @Override
        public <T extends Event> void addListener(boolean receiveCanceled, Consumer<T> consumer) {
            listen(null, consumer);
        }

        @Override
        public <T extends Event> void addListener(boolean receiveCanceled, Class<T> type, Consumer<T> consumer) {
            listen(type, consumer);
        }

        @Override
        public void unregister(Object target) {}

        @Override
        public <T extends Event> T post(T event) {
            for (Map.Entry<Class<?>, Consumer<Object>> l : new ArrayList<>(listeners)) {
                if (!l.getKey().isInstance(event)) continue;
                try {
                    l.getValue().accept(event);
                } catch (ClassCastException e) {
                    if (l.getKey() != Object.class) Lane3.failed(event.getClass().getSimpleName(), e);
                } catch (Throwable t) {
                    Lane3.failed(event.getClass().getSimpleName(), t);
                }
            }
            return event;
        }

        @Override
        public void start() {}

        public int size() {
            return listeners.size();
        }
    }

    /** The game's bus ({@code NeoForge.EVENT_BUS}). */
    public static final Bus GAME = new Bus();

    /** Each mod's own bus. */
    static final Map<String, Bus> MOD_BUSES = new LinkedHashMap<>();

    /** What the mods registered: how to make each value, and the values made. */
    private static final Map<ResourceKey<?>, Supplier<?>> DEFINED = new HashMap<>();
    private static final Map<ResourceKey<?>, Object> MADE = new HashMap<>();
    /** Registrations per registry, for the report. */
    static final Map<String, Integer> REGISTRIES = new LinkedHashMap<>();

    static void clear() {
        DEFINED.clear();
        MADE.clear();
        REGISTRIES.clear();
        MOD_BUSES.clear();
    }

    public static void define(ResourceKey<?> key, Supplier<?> make) {
        DEFINED.put(key, make);
    }

    public static void registered(String registry, int count) {
        REGISTRIES.merge(registry, count, Integer::sum);
        Lane3.registered("registry " + registry);
    }

    public static boolean has(ResourceKey<?> key) {
        return DEFINED.containsKey(key);
    }

    /** The value of a mod's registry entry, made now if it was not yet. */
    public static Object value(ResourceKey<?> key) {
        Object made = MADE.get(key);
        if (made != null) return made;
        Supplier<?> make = DEFINED.get(key);
        if (make == null) throw new IllegalStateException("not registered: " + key);
        made = make.get();
        MADE.put(key, made);
        if (made instanceof net.minecraft.world.level.block.Block b) b.horizonId(key.location().toString());
        return made;
    }

    /** Whether some mod registered {@code id} in the registry named {@code registry}. */
    public static ResourceKey<?> key(String registry, String id) {
        for (ResourceKey<?> k : DEFINED.keySet()) {
            if (k.registry().toString().equals(registry) && k.location().toString().equals(id)) return k;
        }
        return null;
    }

    /** Constructs a mod's {@code @Mod} class with its event bus (and its container, if it asks). */
    static void construct(String id, Class<?> type) throws Throwable {
        Bus bus = MOD_BUSES.computeIfAbsent(id, i -> new Bus());
        ModContainer container = new ModContainer(id, bus);
        try {
            type.getDeclaredConstructor(IEventBus.class, ModContainer.class).newInstance(bus, container);
            return;
        } catch (NoSuchMethodException e) {
            // Another shape.
        }
        try {
            type.getDeclaredConstructor(IEventBus.class).newInstance(bus);
            return;
        } catch (NoSuchMethodException e) {
            // Another shape.
        }
        try {
            type.getDeclaredConstructor(ModContainer.class).newInstance(container);
            return;
        } catch (NoSuchMethodException e) {
            // Another shape.
        }
        type.getDeclaredConstructor().newInstance();
    }

    /** Puts an {@code @EventBusSubscriber} class's static listeners on the bus it names. */
    static void subscribe(String id, Class<?> type) {
        EventBusSubscriber.Bus which = EventBusSubscriber.Bus.GAME;
        try {
            EventBusSubscriber a = type.getAnnotation(EventBusSubscriber.class);
            if (a != null) which = a.bus();
        } catch (Throwable t) {
            // The game's bus, as most are.
        }
        Bus bus = which == EventBusSubscriber.Bus.MOD ? MOD_BUSES.computeIfAbsent(id, i -> new Bus()) : GAME;
        bus.register(type);
    }

    /** What the mods registered, for the report: {@code registry xN, ...}. */
    static String registrations() {
        StringBuilder out = new StringBuilder();
        for (Map.Entry<String, Integer> e : REGISTRIES.entrySet()) {
            if (out.length() > 0) out.append(", ");
            String name = e.getKey();
            out.append(name.substring(name.indexOf(':') + 1)).append(" x").append(e.getValue());
        }
        out.append(out.length() > 0 ? ", " : "").append("game events x").append(GAME.size());
        return out.toString();
    }
}
