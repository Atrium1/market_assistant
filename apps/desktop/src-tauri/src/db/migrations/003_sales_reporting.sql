ALTER TABLE sales ADD COLUMN payment_method TEXT NOT NULL DEFAULT 'cash'
    CHECK (payment_method IN ('cash', 'card'));
ALTER TABLE sales ADD COLUMN business_day TEXT NOT NULL DEFAULT '';
ALTER TABLE sales ADD COLUMN receipt_no INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sales ADD COLUMN voided_at TEXT;
ALTER TABLE sales ADD COLUMN void_reason TEXT;

-- Existing rows: business day = local date of the machine running the upgrade.
UPDATE sales SET business_day = date(paid_at, 'localtime') WHERE business_day = '';

UPDATE sales SET receipt_no = (
    SELECT rn FROM (
        SELECT id, ROW_NUMBER() OVER (
            PARTITION BY store_id, business_day ORDER BY paid_at, id
        ) AS rn
        FROM sales
    ) t
    WHERE t.id = sales.id
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_sales_receipt
    ON sales (store_id, business_day, receipt_no);

CREATE INDEX IF NOT EXISTS idx_sales_day_status
    ON sales (store_id, business_day, status);

CREATE INDEX IF NOT EXISTS idx_sale_items_product
    ON sale_items (product_id);
