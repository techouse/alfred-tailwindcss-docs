class SearchResultHierarchy {
  const SearchResultHierarchy({
    required this.lvl0,
    this.lvl1,
    this.lvl2,
    this.lvl3,
    this.lvl4,
    this.lvl5,
    this.lvl6,
  });

  final String lvl0;
  final String? lvl1;
  final String? lvl2;
  final String? lvl3;
  final String? lvl4;
  final String? lvl5;
  final String? lvl6;

  String? getLevel(int level) {
    if (level == 1) return lvl1;
    if (level == 2) return lvl2;
    if (level == 3) return lvl3;
    if (level == 4) return lvl4;
    if (level == 5) return lvl5;
    if (level == 6) return lvl6;

    return lvl0;
  }

  SearchResultHierarchy.fromJson(Map<String, dynamic> json)
      : lvl0 = json['lvl0'] as String,
        lvl1 = json['lvl1'] as String?,
        lvl2 = json['lvl2'] as String?,
        lvl3 = json['lvl3'] as String?,
        lvl4 = json['lvl4'] as String?,
        lvl5 = json['lvl5'] as String?,
        lvl6 = json['lvl6'] as String?;

  Map<String, String?> toJson() => {
        'lvl0': lvl0,
        'lvl1': lvl1,
        'lvl2': lvl2,
        'lvl3': lvl3,
        'lvl4': lvl4,
        'lvl5': lvl5,
        'lvl6': lvl6,
      };
}
