export function costsColumnsDefinition(intl: IntlShape) {
  return [
    {
      id: "product_category",
      header: intl.formatMessage({ id: "common.product_category" }),
      accessorKey: "productCategory",
      cell: ({ row }) => intl.formatMessage({ id: `product_category.${row.original.productCategoryCode}` }),
    },
    {
      id: "project_description",
      header: intl.formatMessage({ id: "common.project_description" }),
      accessorKey: "projectDescription",
    },
  ] as const satisfies readonly ColumnDef<TransformedCost>[];
}
