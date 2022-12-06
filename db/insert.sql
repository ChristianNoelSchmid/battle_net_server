INSERT INTO evidence_cards (cat_id, item_name)
VALUES 
    (1, "Elf"), 
    (1, "The Polar Express"), 
    (1, "The Grinch"), 
    (2, "Holiday Cake"), 
    (2, "Ugly Christmas Sweater"),
    (2, "Ornament Barrage"),
    (3, "Chris"),
    (3, "Alyssa Q."),
    (3, "Andrea"),
    (3, "Alyssa C."),
    (3, "Kunane");

INSERT INTO users (card_id, user_name, passwd)
VALUES 
    (7, "Chris",  "ChrisSchmid"),
    (8, "Alyssa Q.", "AlyssaSchmid"),
    (9, "Andrea", "AndreaBuckalew"),
    (10, "Alyssa C.", "AlyssaHillen"),
    (11, "Kunane", "KunaneHillen");

INSERT INTO categories (cat_name)
VALUES ("Movies"), ("Murder Weapon"), ("People");
