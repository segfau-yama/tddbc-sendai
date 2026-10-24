package molkky;

record Skittle(int number) {
    Skittle {
        // 存在しない番号のスキットルを生成できないよう、ここで範囲を保証する。
        if (number < 1 || number > 12) {
            throw new IllegalArgumentException("スキットル番号は1〜12です: " + number);
        }
    }
}
