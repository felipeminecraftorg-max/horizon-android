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
        OWN.clear();
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
    @SuppressWarnings("unchecked")
    public static Object value(ResourceKey<?> key) {
        Object made = MADE.get(key);
        if (made != null) return made;
        Supplier<?> make = DEFINED.get(key);
        if (make == null) throw new IllegalStateException("not registered: " + key);
        made = make.get();
        MADE.put(key, made);
        if (made instanceof net.minecraft.world.level.block.Block b) {
            b.horizonId(key.location().toString());
            if (key.isFor(net.minecraft.core.registries.Registries.BLOCK)) {
                ((SimpleRegistry<net.minecraft.world.level.block.Block>) net.minecraft.core.registries.BuiltInRegistries.BLOCK)
                        .hold(key.location(), b);
            }
        }
        if (made instanceof net.minecraft.world.item.Item i && key.isFor(net.minecraft.core.registries.Registries.ITEM)) {
            ((SimpleRegistry<net.minecraft.world.item.Item>) net.minecraft.core.registries.BuiltInRegistries.ITEM)
                    .hold(key.location(), i);
        }
        return made;
    }

    /** Whether some mod registered {@code id} in the registry named {@code registry}: its key. */
    @SuppressWarnings({"unchecked", "rawtypes"})
    public static ResourceKey<?> key(String registry, String id) {
        ResourceKey<?> k;
        try {
            k = ResourceKey.create((ResourceKey) ResourceKey.createRegistryKey(net.minecraft.resources.ResourceLocation.parse(registry)),
                    net.minecraft.resources.ResourceLocation.parse(id));
        } catch (RuntimeException e) {
            return null;
        }
        return DEFINED.containsKey(k) ? k : null;
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

    /** Whether a block class has its own method by a name (asked once a class and name). */
    private static final Map<Class<?>, Map<String, Boolean>> OWN = new HashMap<>();

    /** Whether a mod registered the block {@code id}. */
    public static boolean ownsBlock(String id) {
        return key("minecraft:block", id) != null;
    }

    /**
     * Whether the mod block {@code block} declares its own {@code name} (a class whose methods
     * can't be listed, as one naming a class nobody has, counts as having it: the call then
     * reaches the game's empty one).
     */
    private static synchronized boolean own(Object block, String name) {
        Map<String, Boolean> names = OWN.computeIfAbsent(block.getClass(), c -> new HashMap<>());
        Boolean known = names.get(name);
        if (known != null) return known;
        boolean found = false;
        for (Class<?> c = block.getClass(); c != null && c != net.minecraft.world.level.block.Block.class && !found; c = c.getSuperclass()) {
            // A stand-in for one of the game's classes does nothing of its own.
            if (c.getName().startsWith("net.minecraft.")) continue;
            try {
                for (Method d : c.getDeclaredMethods()) {
                    if (d.getName().equals(name) && !d.isBridge()) {
                        found = true;
                        break;
                    }
                }
            } catch (Throwable t) {
                found = true;
            }
        }
        names.put(name, found);
        return found;
    }

    /** The mod's block of the state {@code state} ({@code places:x[a=b]}), or null. */
    private static net.minecraft.world.level.block.Block block(String state) {
        int bracket = state.indexOf('[');
        ResourceKey<?> key = key("minecraft:block", bracket < 0 ? state : state.substring(0, bracket));
        if (key == null) return null;
        return value(key) instanceof net.minecraft.world.level.block.Block b ? b : null;
    }

    /** An entity (the acting player) is inside the mod block {@code id} at x, y, z. */
    static void entityInside(String id, int x, int y, int z) {
        net.minecraft.world.level.block.Block block = block(id);
        if (block == null || !own(block, "entityInside")) return;
        net.minecraft.world.level.block.state.BlockBehaviour.horizonInside(block, net.minecraft.world.level.block.state.BlockState.of(id),
                net.minecraft.server.level.ServerLevel.CURRENT, new net.minecraft.core.BlockPos(x, y, z), net.minecraft.server.level.ServerPlayer.ACTING);
    }

    /** A result as the game takes it: 0 pass, 1 used (swing), 2 used, 3 refused. */
    private static int code(net.minecraft.world.InteractionResult r) {
        if (r == null || r == net.minecraft.world.InteractionResult.PASS) return 0;
        if (r == net.minecraft.world.InteractionResult.FAIL) return 3;
        return r.shouldSwing() ? 1 : 2;
    }

    /**
     * A player right-clicked the block at x, y, z with {@code hand} (0 main, 1 off), as
     * {@code ServerPlayerGameMode.useItemOn}: NeoForge's event, then a mod's block's own
     * {@code useItemOn} and {@code useWithoutItem}.
     */
    static int use(int x, int y, int z, int face, int handIndex) {
        net.minecraft.world.InteractionHand hand = net.minecraft.world.InteractionHand.values()[handIndex & 1];
        net.minecraft.core.BlockPos pos = new net.minecraft.core.BlockPos(x, y, z);
        net.minecraft.world.phys.BlockHitResult hit =
                new net.minecraft.world.phys.BlockHitResult(pos, net.minecraft.core.Direction.from3DDataValue(face), false);
        net.minecraft.server.level.ServerPlayer player = net.minecraft.server.level.ServerPlayer.ACTING;
        net.neoforged.neoforge.event.entity.player.PlayerInteractEvent.RightClickBlock event =
                new net.neoforged.neoforge.event.entity.player.PlayerInteractEvent.RightClickBlock(player, hand, pos, hit);
        GAME.post(event);
        if (event.isCanceled()) {
            int c = code(event.getCancellationResult());
            return c == 0 ? 3 : c;
        }
        if (event.getUseBlock().isFalse()) return 0;
        String text = Native.block(x, y, z);
        net.minecraft.world.level.block.Block block = text == null ? null : block(text);
        if (block == null) return 0;
        boolean useItemOn = own(block, "useItemOn");
        boolean useWithoutItem = own(block, "useWithoutItem");
        if (!useItemOn && !useWithoutItem) return 0;
        net.minecraft.world.level.block.state.BlockState state = net.minecraft.world.level.block.state.BlockState.of(text);
        net.minecraft.server.level.ServerLevel level = net.minecraft.server.level.ServerLevel.CURRENT;
        boolean holding = !player.getMainHandItem().isEmpty() || !player.getOffhandItem().isEmpty();
        // Sneaking with something in hand skips the block, unless the event asked for it.
        if (player.isShiftKeyDown() && holding && !event.getUseBlock().isTrue()) return 0;
        net.minecraft.world.ItemInteractionResult item = net.minecraft.world.ItemInteractionResult.PASS_TO_DEFAULT_BLOCK_INTERACTION;
        if (useItemOn) {
            item = net.minecraft.world.level.block.state.BlockBehaviour.horizonUseItem(
                    block, player.getItemInHand(hand), state, level, pos, player, hand, hit);
            if (item == null) item = net.minecraft.world.ItemInteractionResult.PASS_TO_DEFAULT_BLOCK_INTERACTION;
            if (item.consumesAction()) return code(item.result());
        }
        if (useWithoutItem && item == net.minecraft.world.ItemInteractionResult.PASS_TO_DEFAULT_BLOCK_INTERACTION
                && hand == net.minecraft.world.InteractionHand.MAIN_HAND) {
            return code(net.minecraft.world.level.block.state.BlockBehaviour.horizonUse(block, state, level, pos, player, hit));
        }
        return 0;
    }

    /** A player broke the mod's block state {@code state} at x, y, z: its {@code onDestroyedByPlayer}. */
    static void broken(String state, int x, int y, int z) {
        net.minecraft.world.level.block.Block block = block(state);
        if (block == null || !own(block, "onDestroyedByPlayer")) return;
        block.onDestroyedByPlayer(net.minecraft.world.level.block.state.BlockState.of(state), net.minecraft.server.level.ServerLevel.CURRENT,
                new net.minecraft.core.BlockPos(x, y, z), net.minecraft.server.level.ServerPlayer.ACTING, true,
                new net.minecraft.world.level.material.FluidState());
    }

    /** {@code Block.onPlace}: the mod's block state {@code state} replaced {@code old} at x, y, z. */
    static void placed(String state, int x, int y, int z, String old) {
        net.minecraft.world.level.block.Block block = block(state);
        if (block == null || !own(block, "onPlace")) return;
        net.minecraft.world.level.block.state.BlockBehaviour.horizonPlace(block, net.minecraft.world.level.block.state.BlockState.of(state),
                net.minecraft.server.level.ServerLevel.CURRENT, new net.minecraft.core.BlockPos(x, y, z),
                net.minecraft.world.level.block.state.BlockState.of(old));
    }

    /** {@code Block.tick}: a tick the mod's block scheduled at x, y, z is due. */
    static void ticked(String state, int x, int y, int z) {
        net.minecraft.world.level.block.Block block = block(state);
        if (block == null || !own(block, "tick")) return;
        net.minecraft.server.level.ServerLevel level = net.minecraft.server.level.ServerLevel.CURRENT;
        net.minecraft.world.level.block.state.BlockBehaviour.horizonTick(block, net.minecraft.world.level.block.state.BlockState.of(state), level,
                new net.minecraft.core.BlockPos(x, y, z), level.random);
    }

    /**
     * The mods' blocks (each made now, as registering them makes them in the game) with their own
     * {@code onPlace} or {@code tick}: {@code place\tid} and {@code tick\tid} lines.
     */
    static String hooks() {
        StringBuilder out = new StringBuilder();
        for (ResourceKey<?> key : new java.util.ArrayList<>(DEFINED.keySet())) {
            if (!key.isFor(net.minecraft.core.registries.Registries.BLOCK)) continue;
            Object block;
            try {
                block = value(key);
            } catch (Throwable t) {
                Lane3.failed("block " + key.location(), t);
                continue;
            }
            String id = key.location().toString();
            if (own(block, "onPlace")) out.append("place\t").append(id).append('\n');
            if (own(block, "tick")) out.append("tick\t").append(id).append('\n');
        }
        return out.toString();
    }

    /** The biome {@code id}'s holder. */
    @SuppressWarnings("unchecked")
    public static net.minecraft.core.Holder<net.minecraft.world.level.biome.Biome> biome(String id) {
        return named((ResourceKey<net.minecraft.world.level.biome.Biome>) (ResourceKey<?>) ResourceKey.create(
                net.minecraft.core.registries.Registries.BIOME, net.minecraft.resources.ResourceLocation.parse(id)));
    }

    /** A level's key by its id: one key per id, as the game's. */
    @SuppressWarnings("unchecked")
    public static ResourceKey<net.minecraft.world.level.Level> level(String id) {
        return (ResourceKey<net.minecraft.world.level.Level>) (ResourceKey<?>) ResourceKey.create(
                net.minecraft.core.registries.Registries.DIMENSION, net.minecraft.resources.ResourceLocation.parse(id));
    }

    /** The acting player: x, y, z, yaw, pitch and its level's id (Native.player). */
    public static String[] player() {
        String p = Native.player();
        return p == null || p.isEmpty() ? new String[0] : p.split(" ");
    }

    /** A registry entry's id ({@code minecraft:blindness}), or "" for one with none. */
    public static String id(net.minecraft.core.Holder<?> holder) {
        if (holder == null) return "";
        return holder.unwrapKey().map(k -> k.location().toString()).orElse("");
    }

    private static final Map<ResourceKey<?>, net.minecraft.core.Holder<?>> NAMED = new HashMap<>();

    /** The game's own entry {@code key} (its value is Horizon's, not the mod's). */
    @SuppressWarnings("unchecked")
    public static synchronized <T> net.minecraft.core.Holder<T> named(ResourceKey<T> key) {
        return (net.minecraft.core.Holder<T>) NAMED.computeIfAbsent(key, k -> new net.minecraft.core.Holder<T>() {
            public T value() {
                return null;
            }

            public boolean isBound() {
                return true;
            }

            public boolean is(net.minecraft.resources.ResourceLocation id) {
                return key.location().equals(id);
            }

            public boolean is(ResourceKey<T> other) {
                return key.equals(other);
            }

            public java.util.Optional<ResourceKey<T>> unwrapKey() {
                return java.util.Optional.of(key);
            }
        });
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
