extern crate treexml;
extern crate treexml_util;

use errors;

use self::treexml_util::Unmarshaller;

use self::treexml_util::{make_text_element, make_tree_element};

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
    pub d_boinc: f64,
    pub d_allowed: f64,

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
    pub fn try_from(root: &treexml::Element) -> Result<HostInfo, errors::Error> {
        let mut v = HostInfo::default();
        for node in &root.children {
            match &*node.name {
                "p_fpops" => {
                    let _ = v.p_fpops.unmarshal(&node);
                }
                "p_iops" => {
                    let _ = v.p_iops.unmarshal(&node);
                }
                "p_membw" => {
                    let _ = v.p_membw.unmarshal(&node);
                }
                "p_calculated" => {
                    let _ = v.p_calculated.unmarshal(&node);
                }
                "p_vm_extensions_disabled" => {
                    let _ = v.p_vm_extensions_disabled.unmarshal(&node);
                }
                "host_cpid" => {
                    let _ = v.host_cpid.unmarshal(&node);
                }
                "product_name" => {
                    let _ = v.product_name.unmarshal(&node);
                }
                "mac_address" => {
                    let _ = v.mac_address.unmarshal(&node);
                }
                "domain_name" => {
                    let _ = v.domain_name.unmarshal(&node);
                }
                "ip_addr" => {
                    let _ = v.ip_addr.unmarshal(&node);
                }
                "p_vendor" => {
                    let _ = v.p_vendor.unmarshal(&node);
                }
                "p_model" => {
                    let _ = v.p_model.unmarshal(&node);
                }
                "os_name" => {
                    let _ = v.os_name.unmarshal(&node);
                }
                "os_version" => {
                    let _ = v.os_version.unmarshal(&node);
                }
                "p_features" => {
                    let _ = v.p_features.unmarshal(&node);
                }
                "serialnum" => {
                    let _ = v.serialnum.unmarshal(&node);
                }
                "timezone" => {
                    let _ = v.tz_shift.unmarshal(&node);
                }
                "p_ncpus" => {
                    let _ = v.p_ncpus.unmarshal(&node);
                }
                "m_nbytes" => {
                    let _ = v.m_nbytes.unmarshal(&node);
                }
                "m_cache" => {
                    let _ = v.m_cache.unmarshal(&node);
                }
                "m_swap" => {
                    let _ = v.m_swap.unmarshal(&node);
                }
                "d_total" => {
                    let _ = v.d_total.unmarshal(&node);
                }
                "d_free" => {
                    let _ = v.d_free.unmarshal(&node);
                }
                _ => {}
            }
        }

        Ok(v)
    }
}
