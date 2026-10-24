package molkky;

import java.util.HashSet;
import java.util.Set;

final class Throw {
    private final Set<Skittle> skittles;

    public Throw(Skittle... fallenSkittles) {
        if (fallenSkittles == null) {
            throw new IllegalArgumentException("スキットルの配列はnullにできません");
        }
        Set<Skittle> validated = new HashSet<>();
        for (Skittle skittle : fallenSkittles) {
            if (skittle == null) {
                throw new IllegalArgumentException("スキットルはnullにできません");
            }
            // 同じ番号のスキットルは1本だけなので、重複を黙って除去せず拒否する。
            if (!validated.add(skittle)) {
                throw new IllegalArgumentException("スキットル番号が重複しています: " + skittle.number());
            }
        }
        skittles = Set.copyOf(validated);
    }

    public int calcPoint() {
        // 1本なら番号が点数になり、0本・複数本なら本数が点数になる。
        return skittles.size() == 1 ? skittles.iterator().next().number() : skittles.size();
    }
}
