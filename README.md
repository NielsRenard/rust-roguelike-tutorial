# Section 1 - Hello Rust
* [x] 1.1. Entities and Components
* [x] 1.2. Walking A Map
* [x] 1.3. A More Interesting Map
* [x] 1.4. Field of View
* [x] 1.5. Monsters
* [x] 1.6. Dealing Damage
* [x] 1.7. User Interface
* [x] 1.8. Items and Inventory
* [ ] 1.9. Ranged Scrolls/Targeting
* [ ] 1.10. Saving and Loading
* [ ] 1.11. Delving Deeper
* [ ] 1.12. Difficulty
* [ ] 1.13. Equipment
# Section 2 - Stretch Goals
* [ ] 2.1. Nice Walls with Bitsets
* [ ] 2.2. Bloodstains
* [ ] 2.3. Particle Effects
* [ ] 2.4. Hunger Clock
* [ ] 2.5. Magic Mapping
* [ ] 2.6. REX Paint Menu
* [ ] 2.7. Simple Traps
# Section 3 - Generating Maps
* [ ] 3.1. Refactor Map Building
* [ ] 3.2. Map Building Test Harness
* [ ] 3.3. BSP Room Dungeons
* [ ] 3.4. BSP Interior Design
* [ ] 3.5. Cellular Automata Maps
* [ ] 3.6. Drunkard's Walk Maps
* [ ] 3.7. Mazes and Labyrinths
* [ ] 3.8. Diffusion-limited aggregation maps
* [ ] 3.9. Add symmetry and brushes to the library
* [ ] 3.10. Voronoi Hive Maps
* [ ] 3.11. Wave Function Collapse
* [ ] 3.12. Prefabs & Sectionals
* [ ] 3.13. Room Vaults
* [ ] 3.14. Layering/Builder Chaining
* [ ] 3.15. Fun With Layers
* [ ] 3.16. Room Builders
* [ ] 3.17. Better Corridors
* [ ] 3.18. Doors
* [ ] 3.19. Decouple map size from screen size
* [ ] 3.20. Section 3 Conclusion
# Section 4 - Making A Game
* [ ] 4.1. Design Document
* [ ] 4.2. Raw Files, Data-Driven Design
* [ ] 4.3. Data-Driven Spawn Tables
* [ ] 4.4. Making the town
* [ ] 4.5. Populating the town
* [ ] 4.6. Living bystanders
* [ ] 4.7. Game Stats
* [ ] 4.8. Equipment
* [ ] 4.9. User Interface
* [ ] 4.10. Into the Woods!
* [ ] 4.11. XP
* [ ] 4.12. Backtracking
* [ ] 4.13. Into the caverns
* [ ] 4.14. Better AI
* [ ] 4.15. Item Stats and Vendors
* [ ] 4.16. Deep caverns
* [ ] 4.17. Cavern to Dwarf Fort
* [ ] 4.18. Town Portals
* [ ] 4.19. Magic Items
* [ ] 4.20. Effects
* [ ] 4.21. Cursed Items
* [ ] 4.22. Even More Items
* [ ] 4.23. Magic Spells
* [ ] 4.24. Enter the Dragon
* [ ] 4.25. Mushrooms
* [ ] 4.26. More Shrooms

[Link to supported glyphs (for fn to_cp437)](https://docs.rs/rltk/0.5.15/src/rltk/codepage437.rs.html#2-276)

["Use usize and isize when it’s related to memory size – the size of an object, or indexing a vector, for instance. It will be a 32-bit number on 32-bit platforms, as that’s the limit of memory they can address, and likewise for 64-bit.
Use u32 and i32 when you just want numbers."](https://users.rust-lang.org/t/i32-vs-isize-u32-vs-usize/22657/3)
