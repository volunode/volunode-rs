extern crate treexml;
extern crate treexml_util;

use errors;

use self::treexml_util::Unmarshaller;

use self::treexml_util::{make_tree_element, make_text_element};

#[derive(Clone, Debug, Default)]
pub struct HostInfo {
    pub tz_shift: i64,
    pub domain_name: String,
    pub serialnum: String,
    pub ip_addr: String,
    pub host_cpid: String,

    pub p_ncpus: i64,
    pub p_vendor: String,
    pub p_model: String,
    pub p_features: String,
    pub p_fpops: f64,
    pub p_iops: f64,
    pub p_membw: f64,
    pub p_calculated: f64,
    pub p_vm_extensions_disabled: bool,

    pub m_nbytes: f64,
    pub m_cache: f64,
    pub m_swap: f64,

    pub d_total: f64,
    pub d_free: f64,

    pub os_name: String,
    pub os_version: String,
    pub product_name: String,

    pub mac_address: String,
}

impl<'a> From<&'a HostInfo> for treexml::Element {
    fn from(v: &HostInfo) -> treexml::Element {
        make_tree_element(
            "host_info",
            vec![
                make_text_element("tz_shift", &v.tz_shift),
                make_text_element("domain_name", &v.domain_name),
                make_text_element("serialnum", &v.serialnum),
                make_text_element("ip_addr", &v.ip_addr),
                make_text_element("host_cpid", &v.host_cpid),
                make_text_element("p_ncpus", &v.p_ncpus),
                make_text_element("p_vendor", &v.p_vendor),
                make_text_element("p_model", &v.p_model),
                make_text_element("p_features", &v.p_features),
                make_text_element("p_fpops", &v.p_fpops),
                make_text_element("p_iops", &v.p_iops),
                make_text_element("p_membw", &v.p_membw),
                make_text_element("p_calculated", &v.p_calculated),
                make_text_element("p_vm_extensions_disabled", &v.p_vm_extensions_disabled),
                make_text_element("m_nbytes", &v.m_nbytes),
                make_text_element("m_cache", &v.m_cache),
                make_text_element("m_swap", &v.m_swap),
                make_text_element("d_total", &v.d_total),
                make_text_element("d_free", &v.d_free),
                make_text_element("os_name", &v.os_name),
                make_text_element("os_version", &v.os_version),
                make_text_element("product_name", &v.product_name),
                make_text_element("mac_address", &v.mac_address),
            ],
        )
    }
}

impl HostInfo {
    pub fn try_from(root: &treexml::Element) -> errors::Result<HostInfo> {
        let mut v = HostInfo::default();
        for node in &root.children {
            v.p_fpops.unmarshal("p_fpops", &node);
            v.p_iops.unmarshal("p_iops", &node);
            v.p_membw.unmarshal("p_membw", &node);
            v.p_calculated.unmarshal("p_calculated", &node);
            v.p_vm_extensions_disabled.unmarshal(
                "p_vm_extensions_disabled",
                &node,
            );
            v.host_cpid.unmarshal("host_cpid", &node);
            v.product_name.unmarshal("product_name", &node);
            v.mac_address.unmarshal("mac_address", &node);
            v.domain_name.unmarshal("domain_name", &node);
            v.ip_addr.unmarshal("ip_addr", &node);
            v.p_vendor.unmarshal("p_vendor", &node);
            v.p_model.unmarshal("p_model", &node);
            v.os_name.unmarshal("os_name", &node);
            v.os_version.unmarshal("os_version", &node);
            v.p_features.unmarshal("p_features", &node);
            v.serialnum.unmarshal("serialnum", &node);
            v.tz_shift.unmarshal("timezone", &node);
            v.p_ncpus.unmarshal("p_ncpus", &node);
            v.m_nbytes.unmarshal("m_nbytes", &node);
            v.m_cache.unmarshal("m_cache", &node);
            v.m_swap.unmarshal("m_swap", &node);
            v.d_total.unmarshal("d_total", &node);
            v.d_free.unmarshal("d_free", &node);
        }

        Ok(v)
    }
}
